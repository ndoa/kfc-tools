COMPILE_TIME_REFLECTION_INFO_EA = 0x141C64F80
CLIENT_BUILD = 491572
OUTPUT_PATH = f'C:\\RE\\Enshrouded\\reflection_type_info_{CLIENT_BUILD}.json'

from pprint import pprint
import json

PRIMATIVE_REFLECTION_TYPES = {
    0: "None",
    1: "Bool",
    2: "Uint8",
    3: "Sint8",
    4: "Uint16",
    5: "Sint16",
    6: "Uint32",
    7: "Sint32",
    8: "Uint64",
    9: "Sint64",
    0xA: "Float32",
    0xB: "Float64",
    0xC: "Enum",
    0xD: "Bitmask8",
    0xE: "Bitmask16",
    0xF: "Bitmask32",
    0x10: "Bitmask64",
    0x11: "Typedef",
    0x12: "Struct",
    0x13: "StaticArray",
    0x14: "DsArray",
    0x15: "DsString",
    0x16: "DsOptional",
    0x17: "DsVariant",
    0x18: "BlobArray",
    0x19: "BlobString",
    0x1A: "BlobOptional",
    0x1B: "BlobVariant",
    0x1C: "ObjectReference",
    0x1D: "Guid",
}

def read_reflection_enum_field(ea):
    print("processing enum field:0x{:X}".format(ea))
    name_ea = get_qword(ea+0x0)
    name_size = get_qword(ea+0x8)
    name = get_strlit_contents(name_ea, name_size, STRTYPE_C).decode()
    
    value = get_qword(ea+0x10)

    return {
        "name": name,
        "value": value,
    }

def read_reflection_struct_field(ea):
    print("processing struct field:0x{:X}".format(ea))
    name_ea = get_qword(ea+0x0)
    name_size = get_qword(ea+0x8)
    name = get_strlit_contents(name_ea, name_size, STRTYPE_C).decode()
    
    type_ea = get_qword(ea+0x10)
    type = read_reflection_type(type_ea, read_fields=False)
    type_name = type['qualified_name']
    
    data_offset = get_qword(ea+0x18)

    return {
        "name": name,
        "type_name": type_name,
        "data_offset": data_offset,
    }

def read_reflection_type(ea, read_fields=True):
    print("processing type:0x{:X}".format(ea))
    qualified_name_ea = get_qword(ea+0x20)
    qualified_name_size = get_qword(ea+0x28)
    qualified_name = get_strlit_contents(qualified_name_ea, qualified_name_size, STRTYPE_C).decode()
    
    referenced_type_ea = get_qword(ea+0x38)
    referenced_type_name = None
    if referenced_type_ea != 0:   
        referenced_type = read_reflection_type(referenced_type_ea, read_fields=False)
        referenced_type_name = referenced_type['qualified_name']
    
    class_size = get_wide_dword(ea+0x40)
    fields_count = get_wide_dword(ea+0x48)
    primative_type_enum = ida_bytes.get_byte(ea+0x4C)
    
    hash1 = get_wide_dword(ea+0x50)
    hash2 = get_wide_dword(ea+0x54)
    struct_fields_list_ea = get_qword(ea+0x58)
    enum_fields_list_ea = get_qword(ea+0x60)
    
    struct_fields = []
    enum_fields = []
    if read_fields:
        if struct_fields_list_ea != 0:
            for i in range(fields_count):
                struct_fields.append(read_reflection_struct_field(struct_fields_list_ea + (i*0x30)))
                
        if enum_fields_list_ea != 0:
            for i in range(fields_count):
                enum_fields.append(read_reflection_enum_field(enum_fields_list_ea + (i*0x28)))
    
    return {
        "qualified_name": qualified_name,
        "referenced_type_name": referenced_type_name,
        "class_size": class_size,
        "fields_count": fields_count,
        "primative_type": PRIMATIVE_REFLECTION_TYPES[primative_type_enum],
        "hash1": hash1,
        "hash2": hash2,
        "struct_fields": struct_fields,
        "enum_fields": enum_fields
    }

def read_table1(ea, count):
    types = []
    for i in range(count):
        type_ea = get_qword(ea + i*8)
        print("read_table1@0x{:X}, type:0x{:X}".format(ea+i*8, type_ea))
        types.append(read_reflection_type(type_ea))
        
        
    return types

if __name__ == '__main__':
    table1_ea = get_qword(g_compile_time_reflection_info)
    table1_count = get_qword(g_compile_time_reflection_info+8)
    type_table = read_table1(table1_ea, table1_count)
    #pprint(type_table)

    with open(OUTPUT_PATH, 'w', encoding='utf-8') as f:
        json.dump(type_table, f, ensure_ascii=False, indent=4)