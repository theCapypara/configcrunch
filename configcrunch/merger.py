"""
Merger module.
Contains the logic to merge loaded documents.

- resolve_and_merge may be used to resolve and merge $ref entries in documents (as used by YamlConfigDocument).
- load_subdocument may be used to load and merge sub-documents contained in YamlConfigDocuments.

"""
from typing import Union, Type, List

from configcrunch import REF, REMOVE

from typing import TYPE_CHECKING

from configcrunch.interface import IYamlConfigDocument
from configcrunch.loader import load_referenced_document

if TYPE_CHECKING:
    from configcrunch.abstract import YamlConfigDocument


def _merge_documents__recursion(target_node: any, source_node: any) -> any:
    """
    Recursive merging step of merge_documents
    :param target_node: Node to MERGE INTO
    :param source_node: Node to MERGE FROM
    :return: Merge result
    """
    # IS DICT IN SOURCE AND TARGET
    if isinstance(source_node, dict) and isinstance(target_node, dict):
        new_node = target_node.copy()
        for key, value in source_node.items():
            if value == REMOVE:
                del new_node[key]
            else:
                if key in target_node:
                    new_node[key] = _merge_documents__recursion(target_node[key], source_node[key])
                else:
                    new_node[key] = source_node[key]
        return new_node

    # IS LIST IN SOURCE AND TARGET
    elif isinstance(source_node, list) and isinstance(target_node, list):
        old_list = set(source_node)
        add_list = set(target_node)
        return list(old_list | add_list)

    # IS YCD IN SOURCE AND TARGET
    elif isinstance(source_node, IYamlConfigDocument) and isinstance(target_node, IYamlConfigDocument):
        merge_documents(source_node, target_node)
        return source_node

    # IS $remove IN SOURCE AND ... IN TARGET
    if source_node == REMOVE:
        raise Exception("Tried to remove a node at an unexpected position")  # todo exception

    # IS SCALAR IN BOTH (or just in SOURCE)
    else:
        return source_node


def merge_documents(target: 'YamlConfigDocument', source: 'YamlConfigDocument') -> None:
    """
    Merges two YamlConfigDocuments.
    :param target: Target document - this document will be changed, it will contain the result of merging target into source.
    :param source: Source document to base merge on
    """
    newdoc = _merge_documents__recursion(source.doc, target.doc)
    target.doc = newdoc


def resolve_and_merge(doc: 'YamlConfigDocument', lookup_paths: List[str]) -> None:
    """
    Resolve the $ref entry at the beginning of the document body and merge with referenced documents
    (changes this document in place).
    May also be extended by subclasses to include sub-document resolving.
    :param doc: Document to work on
    :param lookup_paths: Paths to the repositories, where referenced should be looked up.
    :return:
    """
    if REF in doc:
        # Resolve references
        prev_referenced_doc = None
        for referenced_doc in load_referenced_document(doc, lookup_paths):
            if prev_referenced_doc:
                # Merge referenced docs
                merge_documents(referenced_doc, prev_referenced_doc)
            prev_referenced_doc = referenced_doc
        if prev_referenced_doc is None:
            raise Exception("Referenced document not found")  # todo
        # Resolve entire referenced docs
        resolve_and_merge(prev_referenced_doc, lookup_paths)
        # Merge content of current doc into referenced doc (and execute $remove's on the way)
        merge_documents(doc, prev_referenced_doc)
        # Remove $ref entry
        del doc[REF]


def load_subdocument(
        doc: 'Union[dict, YamlConfigDocument]',
        doc_path: Union[None, str],
        doc_clss: 'Type[YamlConfigDocument]',
        lookup_paths: List[str]
) -> 'YamlConfigDocument':
    """
    Load a subdocument of a specific type. This will convert the dict at this position
    into a YamlConfigDocument with the matching type and perform resolve_and_merge_references
    on it.
    :param doc: Dictionary with data to convert. Can also already be a document of the target type.
    :param doc_path: Path of the parent document.
    :param doc_clss: Class that is expected from the subdocument (target class)
    :param lookup_paths: Paths to the repositories, where referenced should be looked up.
    :return:
    """
    doc_obj = doc
    if not isinstance(doc, doc_clss):
        print("LOADING INTERNAL SUB DOC:")
        doc_obj = doc_clss(doc, doc_path)
    return doc_obj.resolve_and_merge_references(lookup_paths)


"""
Algo (so hab ich ihn hoffentlich auch):

START (Laden einer Projektdatei):
    project:
    --> $ref: {pfad}
        --> REF-AUFLÖSUNG
    --> app: {...}
        --> LADEN-SUBDOKUMENT(Typ)


REF-AUFLÖSUNG:
  --> Für ref.-Dokument in LADE-REFERENZIERTES-DOKUMENT:
      --> Falls vorheriges-ref.-Dokument:
          --> MERGE(ref.Dokument<--vorheriges-ref.-Dokument)
      --> vorheriges-ref.-Dokument = ref.Dokument
  --> REF-AUFLÖSUNG für vorheriges-ref.-Dokument
  --> MERGE(Eingabe-Dokument<--vorheriges-ref.-Dokument)


LADEN-SUBDOKUMENT(Typ)
  --> ERSTELLEN eines Dokument vom Typ
      --> START auf Dokument

LADE-REFERENZIERTES-DOKUMENT:
  --> pfad_im_repo = Pfad absolut im Repo auflösen, je nachdem wo Quelldokument her kam
  --> Für alle (absoluten Repopfade + pfad_im_repo):
      --> Für alle möglichen Dateien im Repo (.yml,.yaml...)
          --> ERSTELLEN eines Dokument vom Typ des original dokumentes
  --> Alle Dokumente zurückgeben

MERGE(Ziel<--Quelle):
  --> Auf Quelle:
      --> Überschreibe jedes Blatt(=nicht list/dict) der in Ziel vorkommt in Quelle mit dem Ziel-Blatt.
            Besondere Fälle:
            --> Falls Zielknoten == $remove: Entferne Knoten aus Quelle
            --> Falls Zielknoten ein Unterdokument ist, führe MERGE(Ziel.Unterdokument<--Quelle.Unterdokument) aus.

"""