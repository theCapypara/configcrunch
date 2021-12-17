from configcrunch import YamlConfigDocument


# Called from Rust code (YamlConfigDocument::to_dict).
def frozen_ycd_to_dict(ycd: YamlConfigDocument):
    return {ycd.header(): frozen_ycd_to_dict_rec(ycd)}


def frozen_ycd_to_dict_rec(input):
    """ Recursively removes all YamlConfigDocuments and replaces them by their doc dictionary."""
    if isinstance(input, YamlConfigDocument):
        return frozen_ycd_to_dict_rec(input.doc.copy())
    elif isinstance(input, dict):
        new_dict = input.copy()
        for key, val in new_dict.items():
            new_dict[key] = frozen_ycd_to_dict_rec(val)
        return new_dict
    elif isinstance(input, list):
        new_list = []
        for item in input.copy():
            new_list.append(frozen_ycd_to_dict_rec(item))
        return new_list
    return input
