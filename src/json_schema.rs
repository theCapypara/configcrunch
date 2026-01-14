use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::exceptions;
use pyo3::types::{PyDict, PyList, PyType};

use crate::{YamlConfigDocument, DocReference, JsonSchemaDefinitionNotFoundError, JsonSchemaMultiReferenceError};

// NOTE: This is specific to draft-07 JSON schemas, starting with 2019-09 "$defs" should be used instead.
const DEFINITIONS_KEYWORD: &str = "definitions";
const JSON_SCHEMA_DIALECT: &str = "draft-07";

#[pymethods]
impl YamlConfigDocument {
    #[classmethod]
    pub(crate) fn json_schema<'py>(_cls: Bound<'py, PyType>, main_schema_id: String, py: Python<'py>) -> PyResult<HashMap<String, Bound<'py, PyDict>>> {
        let mut json_schemas: HashMap<String, Bound<PyDict>> = HashMap::new();
        let mut schema_id_map: HashMap<String, Option<String>> = HashMap::new();
        // Get the main schema object
        let schema = _cls.getattr("schema")?.call0()?;
        // Recursively replace all DocReferences
        let modified_schema = replace_refs_with_schema(schema, py, &mut schema_id_map)?;
        let json_schema = modified_schema
            .getattr("json_schema")?
            .call1((&main_schema_id, ))?
            .cast_into::<PyDict>()?;
        json_schemas.insert(main_schema_id.clone(), json_schema);

        // The generated schema doesn't need to be modified
        if schema_id_map.is_empty() {
            return Ok(json_schemas);
        }

        // Get the JSON Schema dialect of the generated schema
        let Some(dialect_value) = json_schemas[&main_schema_id].get_item("$schema")? else {
            return Err(exceptions::PyKeyError::new_err(
                "No JSON Schema dialect was specified for the generated schema."
            ));
        };
        let dialect_value: String = dialect_value.extract()?;

        // TODO: Match $schema for: http(s?)://json-schema.org/draft-07/schema(#?)

        let Some(definitions) = json_schemas[&main_schema_id].get_item(DEFINITIONS_KEYWORD)? else {
            return Err(exceptions::PyKeyError::new_err(format!(
                "{} container not found. Was this schema generated for the {} JSON Schema dialect?",
                DEFINITIONS_KEYWORD,
                JSON_SCHEMA_DIALECT
            )));
        };
        let definitions = definitions.cast_into::<PyDict>()?;

        // Rewrite $refs
        replace_ref_values(&json_schemas[&main_schema_id], &schema_id_map)?;

        // Add mapped schemas to result map
        map_definitions_to_schemas(&dialect_value, &mut json_schemas, &definitions, &schema_id_map)?;

        // Copy remaining local definitions to all other schemas
        if ! definitions.is_empty() {
            let definitions = definitions.copy()?;
            json_schemas[&main_schema_id].del_item(DEFINITIONS_KEYWORD)?;
            copy_required_definitions_to_schemas(&json_schemas, &definitions, py)?;
        }
        else {
            // Remove empty definitions container
            json_schemas[&main_schema_id].del_item(DEFINITIONS_KEYWORD)?;
        }

        // Improve self references
        for (schema_id, schema_def) in &json_schemas {
            update_self_refs(schema_def, schema_id)?;
        }

        Ok(json_schemas)
    }
}

/// Replaces all DocReference objects in the specified schema with schema objects.
/// Returns the modified schema.
/// Mappings for a referenced schema name to its corresponding json_schema_id (when it contains a value) will be added to schema_id_map.
fn replace_refs_with_schema<'py>(schema: Bound<'py, PyAny>, py: Python<'py>, schema_id_map: &mut HashMap<String, Option<String>>) -> PyResult<Bound<'py, PyAny>> {
    let schema_class = PyModule::import(py, "schema")?.getattr("Schema")?;
    if schema.is_instance(&schema_class)? {
        let child_schema = schema.getattr("schema")?;
        let child_schema = replace_refs_with_schema(child_schema, py, schema_id_map)?;
        if ! child_schema.is_instance_of::<PyDict>() {
            // Replace the internal _schema attribute
            schema.setattr("_schema", child_schema)?;
        }
    }
    else if schema.is_instance_of::<PyDict>() {
        let schema_dict = schema.cast_into::<PyDict>()?;
        for (key, value) in schema_dict.iter() {
            let new_value = replace_refs_with_schema(value, py, schema_id_map)?;
            schema_dict.set_item(key, new_value)?;
        }

        return Ok(schema_dict.into_any());
    }
    else if schema.is_instance_of::<PyList>() {
        let schema_array = schema.cast_into::<PyList>()?;
        for index in  0..schema_array.len() {
            let item = schema_array.get_item(index)?;
            let new_item = replace_refs_with_schema(item, py, schema_id_map)?;
            schema_array.set_item(index, new_item)?;
        }

        return Ok(schema_array.into_any());
    }
    else if schema.is_instance_of::<DocReference>() {
        let doc_ref: Bound<DocReference> = schema.extract()?;
        let doc_schema = get_schema_for_ref(doc_ref, py, schema_id_map)?;

        return replace_refs_with_schema(doc_schema, py, schema_id_map);
    }

    // Return the (modified) object
    return Ok(schema);
}

fn get_schema_for_ref<'py>(doc_ref: Bound<'py, DocReference>, py: Python<'py>, schema_id_map: &mut HashMap<String, Option<String>>) -> PyResult<Bound<'py, PyAny>> {
    let schema_class = PyModule::import(py, "schema")?.getattr("Schema")?;
    let doc_schema = doc_ref.borrow()
        .referenced_type
        .extract::<Bound<PyType>>(py)?
        .getattr("schema")?
        .call0()?;

    let schema_name: String = doc_schema.getattr("name")?.extract()?;
    let schema_id = doc_ref.borrow().json_schema_id.clone();

    let kwargs = PyDict::new(py);
    // Ignore Schema.error, since it's not used for json schemas
    kwargs.set_item("name", &schema_name)?;
    kwargs.set_item("description", doc_schema.getattr("description")?)?;
    kwargs.set_item("ignore_extra_keys", doc_schema.getattr("ignore_extra_keys")?)?;
    kwargs.set_item("as_reference", true)?;

    // Check whether the schema was already created before
    let inner_schema;
    if ! schema_id_map.contains_key(&schema_name) {
        inner_schema = doc_schema.getattr("schema")?;
    }
    else {
        // Create a dummy schema to prevent infinite recursion
        inner_schema = PyDict::new(py).into_any();
    }
    
    // Add new mapping entry if possible
    let mapped_schema_id = schema_id_map.entry(schema_name.clone()).or_insert(schema_id.clone());
    if schema_id.is_some() && *mapped_schema_id != schema_id {
        return Err(JsonSchemaMultiReferenceError::new_err(format!(
            "Unable to map the schema '{}' to the specified id '{:?}'. The schema is already mapped to: {:?}",
            schema_name,
            schema_id,
            *mapped_schema_id
        )));
    }

    return schema_class.call((inner_schema, ), Some(&kwargs));
}

fn replace_ref_values(json_schema: &Bound<PyDict>, schema_id_map: &HashMap<String, Option<String>>) -> PyResult<()> {
    for (key, value) in json_schema.iter() {
        if value.is_instance_of::<PyDict>() {
            replace_ref_values(&value.cast_into()?, schema_id_map)?;
            continue;
        }
        let key: String = key.extract()?;
        if key == "$ref" {
            let Ok(value) = value.extract::<String>() else {
                continue;
            };
            // NOTE: Specific to draft-07
            let Some(def_name) = value.strip_prefix(&format!("#/{DEFINITIONS_KEYWORD}/")) else {
                return Err(exceptions::PyValueError::new_err(format!(
                    "Unexpected JSON schema reference: {}",
                    value
                )));
            };
            let Some(Some(schema_id)) = schema_id_map.get(def_name) else {
                continue;
            };
            json_schema.set_item(key, schema_id)?
        }
    }
    Ok(())
}

fn map_definitions_to_schemas<'py>(json_schema_dialect: &String, json_schemas: &mut HashMap<String, Bound<'py, PyDict>>, json_definitions: &Bound<'py, PyDict>, schema_id_map: &HashMap<String, Option<String>>) -> PyResult<()> {
    for (schema_name, schema_id) in schema_id_map {
        let Some(schema_id) = schema_id else {
            continue;
        };
        let Some(schema_def) = json_definitions.get_item(&schema_name)? else {
            return Err(JsonSchemaDefinitionNotFoundError::new_err(format!(
                "No schema definition found for: {}",
                schema_name
            )));
        };
        if let Some(existing_schema) = json_schemas.get(schema_id) {
            return Err(JsonSchemaMultiReferenceError::new_err(format!(
                "Unable to use id '{}' for schema '{}'. The id is already used by another schema: {:?}",
                schema_id,
                schema_name,
                existing_schema.get_item("title")?
            )));
        }

        let schema_def = schema_def.cast_into::<PyDict>()?;
        schema_def.set_item("$id", schema_id.clone())?;
        schema_def.set_item("$schema", json_schema_dialect.clone())?;

        json_schemas.insert(schema_id.clone(), schema_def);
        json_definitions.del_item(&schema_name)?;
    }
    Ok(())
}

fn get_local_refs(local_refs: &mut Vec<String>, json_schema: &Bound<PyDict>, json_definitions: &Bound<PyDict>) -> PyResult<()> {
    for (key, value) in json_schema.iter() {
        if value.is_instance_of::<PyDict>() {
            get_local_refs(local_refs, &value.cast_into()?, json_definitions)?;
            continue;
        }
        let key: String = key.extract()?;
        if key == "$ref" {
            let Ok(value) = value.extract::<String>() else {
                continue;
            };
            // NOTE: Specific to draft-07
            let Some(definition_name) = value.strip_prefix(&format!("#/{DEFINITIONS_KEYWORD}/")) else {
                continue;
            };
            let definition_name = definition_name.to_owned();
            
            if ! local_refs.contains(&definition_name) {
                let Some(definition) = json_definitions.get_item(&definition_name)? else {
                    return Err(JsonSchemaDefinitionNotFoundError::new_err(format!(
                        "No schema definition found for: {}",
                        definition_name
                    )));
                };
                
                local_refs.push(definition_name);

                // Check refs in definition too
                get_local_refs(local_refs, &definition.cast_into()?, json_definitions)?;   
            }
        }
    }
    Ok(())
}

fn copy_required_definitions_to_schemas<'py>(json_schemas: &HashMap<String, Bound<'py, PyDict>>, json_definitions: &Bound<'py, PyDict>, py: Python<'py>) -> PyResult<()> {
    let deepcopy_func = PyModule::import(py, "copy")?.getattr("deepcopy")?;
    for (_, schema_def) in json_schemas {
        let mut local_refs: Vec<String> = Vec::new();
        get_local_refs(&mut local_refs, schema_def, json_definitions)?;
        
        if local_refs.is_empty() {
            continue;
        }
        
        // Copy definition entries
        let schema_definitions = PyDict::new(py);
        for definition_name in local_refs {
            let definition = deepcopy_func.call1((json_definitions.get_item(&definition_name)?,))?;
            schema_definitions.set_item(definition_name, definition)?;
        }

        // Assign new definitions dict
        schema_def.set_item(DEFINITIONS_KEYWORD, schema_definitions)?;
    }
    Ok(())
}

fn update_self_refs(json_schema: &Bound<PyDict>, schema_id: &String) -> PyResult<()> {
    for (key, value) in json_schema.iter() {
        if value.is_instance_of::<PyDict>() {
            update_self_refs(&value.cast_into()?, schema_id)?;
            continue;
        }
        let key: String = key.extract()?;
        if key == "$ref" {
            let Ok(value) = value.extract::<String>() else {
                continue;
            };
            if value == *schema_id {
                json_schema.set_item(key, "#")?    
            }  
        }
    }
    Ok(())
}
