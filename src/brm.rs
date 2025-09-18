use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use regex::Regex;
use rand::Rng;
use std::fs;
use std::process::Command;

/// Structure to represent Field (Field Name, Field Type, Field Number)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub data_type: String,
    pub num: String,
}

/// Type aliases for session-based field storage
pub type SessionHashMap = HashMap<String, Field>;
pub type SessionMap = Arc<Mutex<HashMap<String, SessionHashMap>>>;

// Global session map for storing BRM fields per session
lazy_static::lazy_static! {
    pub static ref G_SESSION_MAP: SessionMap = Arc::new(Mutex::new(HashMap::new()));
}

/// Load Oracle BRM fields into session-specific storage
pub fn load_obrm_fields(session_id: &str, data: String) -> String {
    let mut session_map = G_SESSION_MAP.lock().unwrap();
    let field_count = session_map
        .get(session_id)
        .map_or(0, |m| m.len());

    if field_count == 0 {
        let mut new_session_map = HashMap::new();
        for line in data.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 3 {
                new_session_map.insert(parts[0].to_string(), Field {
                    name: parts[0].to_string(),
                    data_type: parts[1].to_string(),
                    num: parts[2].to_string(),
                });
            }
        }
        let response = format!("Fields ({}) loaded!!!", new_session_map.len());
        session_map.insert(session_id.to_owned(), new_session_map);
        response
    } else {
        format!("Fields ({}) already loaded!!!", field_count)
    }
}

/// Get field type for a specific session
pub fn get_field_type(session_id: &str, field_name: &str) -> String {
    let session_map = G_SESSION_MAP.lock().unwrap();
    session_map
        .get(session_id)
        .and_then(|m| m.get(field_name))
        .map(|f| f.data_type.to_string())
        .unwrap_or_else(|| "Field Not Found while getting its data type".to_string())
}

/// Get field number for a specific session
pub fn get_field_num(session_id: &str, field_name: &str) -> i32 {
    let session_map = G_SESSION_MAP.lock().unwrap();
    session_map
        .get(session_id)
        .and_then(|m| m.get(field_name))
        .and_then(|f| f.num.parse::<i32>().ok())
        .unwrap_or(-1)
}

/// Get all fields for a session
pub fn get_session_fields(session_id: &str) -> Vec<Field> {
    let session_map = G_SESSION_MAP.lock().unwrap();
    session_map
        .get(session_id)
        .map(|fields| fields.values().cloned().collect())
        .unwrap_or_default()
}

/// Response structure for load_obrm_fields API
#[derive(Debug, Serialize)]
pub struct LoadFieldsResponse {
    pub success: bool,
    pub message: String,
    pub field_count: usize,
    pub session_id: String,
}

/// Response structure for PODL conversion API
#[derive(Debug, Serialize)]
pub struct PodlConversionResponse {
    pub success: bool,
    pub message: String,
    pub podl_content: String,
}

/// Enumerated type for object creation flag 
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum ObjCreatePermission {
    System,
    Required,
    Prohibited,
    Optional,
}

/// Enumerated type for object modification flag
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum ObjModifyPermission {
    System,
    Writeable,
    NotWriteable,
}

/// Structure to hold the field details for PODL
#[derive(Debug, Clone)]
pub struct FieldPODL {
    pub name: String,
    #[allow(dead_code)]
    pub num: i32,
    pub data_type: String,
    pub descr: String,
    #[allow(dead_code)]
    pub data_length: i32,
    pub create_flag: ObjCreatePermission,
    pub modify_flag: ObjModifyPermission,
    pub audit_flag: bool,
    pub encryp_flag: bool,
    pub nested_fields: Vec<FieldPODL>,
    #[allow(dead_code)]
    pub col_tab_name: String,
}

/// Structure to hold the class details for PODL
#[derive(Debug, Clone)]
pub struct ClassPODL {
    pub type_str: String,
    pub descr: String,
    pub no_of_fields: i32,
    #[allow(dead_code)]
    pub table_name: String,
    pub fields: Vec<FieldPODL>,
    pub fields_0th_lvl: HashMap<String, i8>,
    pub fields_1st_lvl: HashMap<(String, String), i8>,
    pub fields_2nd_lvl: HashMap<(String, String, String), i8>,
}

impl std::fmt::Display for ObjCreatePermission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ObjCreatePermission::System => write!(f, "System"),
            ObjCreatePermission::Required => write!(f, "Required"),
            ObjCreatePermission::Optional => write!(f, "Optional"),
            ObjCreatePermission::Prohibited => write!(f, "Prohibited"),
        }
    }
}

impl std::fmt::Display for ObjModifyPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ObjModifyPermission::System => write!(f, "System"),
            ObjModifyPermission::Writeable => write!(f, "Writeable"),
            ObjModifyPermission::NotWriteable => write!(f, "Not_Writeable"),
        }
    }
}

impl ClassPODL {
    /// Create empty ClassPODL object
    pub fn new() -> ClassPODL {
        ClassPODL {
            type_str: "".to_string(),
            descr: "".to_string(),
            table_name: "".to_string(),
            no_of_fields: -1,
            fields: Vec::new(),
            fields_0th_lvl: HashMap::new(),
            fields_1st_lvl: HashMap::new(),
            fields_2nd_lvl: HashMap::new(),
        }
    }

    /// Create ClassPODL object with initial values
    pub fn new_value(stype: String, sdescr: String, stable_name: String) -> ClassPODL {
        ClassPODL {
            type_str: stype,
            descr: sdescr,
            table_name: stable_name,
            no_of_fields: 0,
            fields: Vec::new(),
            fields_0th_lvl: HashMap::new(),
            fields_1st_lvl: HashMap::new(),
            fields_2nd_lvl: HashMap::new(),
        }
    }

    /// Add a 0th level field to the class
    pub fn add_0th_lvl_field(&mut self, field_name: String, field: FieldPODL) {
        if self.fields_0th_lvl.contains_key(&field_name) {
            return;
        }
        self.no_of_fields += 1;
        self.fields.push(field);
        self.fields_0th_lvl.insert(field_name, 0);
    }

    /// Add a 1st level nested field
    pub fn add_1st_lvl_field(&mut self, field_at_0th: String, field_name: String, field: FieldPODL) {
        if self.fields_1st_lvl.contains_key(&(field_at_0th.clone(), field_name.clone())) {
            return;
        }
        for lfield in self.fields.iter_mut() {
            if lfield.name == field_at_0th {
                lfield.nested_fields.push(field);
                self.fields_1st_lvl.insert((field_at_0th, field_name), 0);
                break;
            }
        }
    }

    /// Add a 2nd level nested field
    pub fn add_2nd_lvl_field(&mut self, field_at_0th: String, field_at_1st: String, field_name: String, field: FieldPODL) {
        if self.fields_2nd_lvl.contains_key(&(field_at_0th.clone(), field_at_1st.clone(), field_name.clone())) {
            return;
        }
        for lfield in self.fields.iter_mut() {
            if lfield.name == field_at_0th {
                for n1field in lfield.nested_fields.iter_mut() {
                    if n1field.name == field_at_1st {
                        n1field.nested_fields.push(field);
                        self.fields_2nd_lvl.insert((field_at_0th, field_at_1st, field_name), 0);
                        return;
                    }
                }
            }
        }
    }

    /// Generate PODL content from the class structure
    pub fn generate_podl(&self) -> Vec<String> {
        let mut podl_content: Vec<String> = Vec::new();

        // Class header
        podl_content.push(format!(
            "#=======================================\n\
             #  Storable Class {}\n\
             #=======================================\n\
             \n\
             STORABLE CLASS {} {{\n\
             \tDESCR = {};\n\
             \tREAD_ACCESS = \"BrandLineage\";\n\
             \tWRITE_ACCESS = \"BrandLineage\";\n\
             \tIS_PARTITIONED = \"0\";\n\
             \tEVENT_TYPE = \"NONE\";",
            self.type_str, self.type_str, self.descr
        ));

        // Fields section
        if !self.fields.is_empty() {
            podl_content.push(format!(
                "\t#=======================================\n\
                 \t#  Fields\n\
                 \t#======================================="
            ));

            for field in &self.fields {
                let mut data_type = field.data_type.trim().to_string();
                if data_type.starts_with("PIN_FLDT_") {
                    data_type = data_type[9..].to_string();
                    if data_type == "STR" {
                        data_type = "STRING".to_string();
                    } else if data_type == "TSTAMP" {
                        data_type = "TIMESTAMP".to_string();
                    }
                }

                podl_content.push(format!(
                    "\t{} {} {{\n\
                     \t\tDESCR = {};\n\
                     \t\tORDER = 0;\n\
                     \t\tCREATE = {};\n\
                     \t\tMODIFY = {};\n\
                     \t\tAUDITABLE = {};\n\
                     \t\tENCRYPTABLE = {};\n\
                     \t\tSERIALIZABLE = 0;\n\
                     \t}}",
                    data_type, field.name, field.descr,
                    field.create_flag, field.modify_flag,
                    field.audit_flag as u8, field.encryp_flag as u8
                ));
            }
        }

        podl_content.push("}".to_string());
        podl_content
    }
}

/// Utility Functions

/// Get the first character from the input string
pub fn get_first_char(s: &str) -> char {
    s.chars().next().unwrap_or(' ')
}

/// Get the number of tab characters at the beginning of a string
pub fn get_begining_tab_count(s: &str) -> i32 {
    let mut count = 0;
    for c in s.chars() {
        if c == '\t' {
            count += 1;
        } else {
            return count;
        }
    }
    count
}

/// Create a FieldPODL instance from field specification data
pub fn create_field_podl(session_id: &str, field_name: &str, data: Vec<&str>) -> FieldPODL {
    let field_name = field_name.trim();
    
    let fname = field_name.to_string();
    let fnum = get_field_num(session_id, field_name);
    let fdata_type = get_field_type(session_id, field_name);
    let fdescr = data.get(0).unwrap_or(&"").trim().to_string();
    let fcol_tab_name = data.get(1).unwrap_or(&"").trim().to_string();

    // Parse flags from data[2] if available
    let flags_str = data.get(2).unwrap_or(&"0,0,0,0").trim();
    let flags: Vec<&str> = flags_str.split(',').collect();

    let fcreate_flag = match flags.get(0).unwrap_or(&"0") {
        &"0" => ObjCreatePermission::Required,
        &"1" => ObjCreatePermission::Optional,
        &"2" => ObjCreatePermission::Prohibited,
        &"3" => ObjCreatePermission::System,
        _ => ObjCreatePermission::Required,
    };

    let fmodify_flag = match flags.get(1).unwrap_or(&"0") {
        &"0" => ObjModifyPermission::Writeable,
        &"1" => ObjModifyPermission::NotWriteable,
        &"2" => ObjModifyPermission::System,
        _ => ObjModifyPermission::Writeable,
    };

    let faudit_flag = match flags.get(2).unwrap_or(&"0") {
        &"0" => false,
        &"1" => true,
        _ => false,
    };

    let fencryp_flag = match flags.get(3).unwrap_or(&"0") {
        &"0" => false,
        &"1" => true,
        _ => false,
    };

    let fdata_length = if fdata_type == "PIN_FLDT_STR" {
        data.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(60)
    } else {
        0
    };

    FieldPODL {
        name: fname,
        num: fnum,
        data_type: fdata_type,
        descr: fdescr,
        data_length: fdata_length,
        create_flag: fcreate_flag,
        modify_flag: fmodify_flag,
        audit_flag: faudit_flag,
        encryp_flag: fencryp_flag,
        nested_fields: Vec::new(),
        col_tab_name: fcol_tab_name,
    }
}

/// Convert custom BRM field specification to PODL format (session-based)
/// 
/// Input format: field_name|description|data_type|field_id
/// Output: PODL format with proper headers and structure
/// 
/// **Session-based validation:**
/// - Requires valid session_id with loaded fields
/// - Validates session exists before processing
pub fn convert_fld_spec_to_podl(session_id: &str, data: String) -> String {
    // Check if session exists and has loaded fields
    let field_count = G_SESSION_MAP.lock().unwrap()
        .get(session_id)
        .map_or(0, |m| m.len());

    if field_count == 0 {
        return "Please load BRM fields first using the load_obrm_fields endpoint !!!".to_string();
    }

    let podl_content = data.lines().map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            format!("#=======================================\n\
                     #  Field {}\n\
                     #=======================================\n\
                     \n\
                     {} {} {{\n\
                     \tID = {};\n\
                     \tDESCR = {};\n\
                     }}\n",
                    parts[0].trim(), parts[2].trim(), parts[0].trim(),
                    parts[3].trim(), parts[1].trim())
        } else {
            format!("# Invalid line format: {}\n", line)
        }
    }).collect::<Vec<String>>().join("\n");
    podl_content
}

// === FLIST CONVERSION FUNCTIONALITY ===

// Defining BRM Data Type mapping for FLIST conversions
lazy_static::lazy_static! {
    static ref BRM_DTYPE_DICT: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert("F".to_string(), "pin_flist_t".to_string());
        map.insert("P".to_string(), "poid_t".to_string());
        map.insert("S".to_string(), "char".to_string());
        map.insert("E".to_string(), "u_int".to_string());
        map.insert("I".to_string(), "int32".to_string());
        map.insert("D".to_string(), "pin_decimal_t".to_string());
        map.insert("T".to_string(), "time_t".to_string());
        map
    };
}

/// Structure to represent a field in FLIST format
#[derive(Clone, Debug)]
pub struct LField {
    pub field_level: i8,
    pub field_name: String,
    pub field_type: String,
    pub field_value: String,
    pub field_arr_idx: String,
}

impl LField {
    /// Create empty LField object
    pub fn new() -> LField {
        LField {
            field_level: 0,
            field_arr_idx: "0".to_string(),
            field_name: "".to_string(),
            field_type: "".to_string(),
            field_value: "".to_string(),
        }
    }

    /// Parse a FLIST line and set field properties
    pub fn set_field(&mut self, line: String) {
        let re = Regex::new(r#"^(\d+)\s+(\S+)\s+(\S+)\s+\[(.*)\](.*)$"#).unwrap();
        if let Some(captures) = re.captures(&line) {
            self.field_level = captures.get(1).unwrap().as_str().parse::<i8>().unwrap();
            self.field_name = captures.get(2).unwrap().as_str().trim().to_string();
            self.field_type = captures.get(3).unwrap().as_str().trim().to_string();
            self.field_arr_idx = captures.get(4).unwrap().as_str().trim().to_string();
            self.field_value = captures.get(5).unwrap().as_str().trim().to_string();
        }

        // Clean up field values based on type
        if self.field_value.contains("NULL str ptr") && self.field_type == "STR" {
            self.field_value = "NULL".to_string();
        }
        if self.field_value.contains("NULL") && self.field_type != "STR" {
            self.field_value = "NULL".to_string();
        }
        if self.field_value.contains("null") && self.field_type == "TSTAMP" {
            self.field_value = "(0)".to_string();
        }
        if !self.field_value.contains("null") && self.field_type == "TSTAMP" {
            let re = Regex::new(r#"(\d+).*$"#).unwrap();
            if let Some(captures) = re.captures(&self.field_value) {
                self.field_value = captures.get(1).unwrap().as_str().trim().to_string();
            }
        }
    }
}

/// Main structure for FLIST to Code/XML/JSON conversion
#[derive(Debug)]
pub struct Flist2Code {
    pub cur_flistp: String,
    pub prev_flistp: String,
    pub cur_field: LField,
    pub tab: String,
    pub vars: String,
    pub code: String,
    pub c_code: String,
    pub xml_code: String,
    pub json_code: String,
    pub cur_level: i8,

    pub xml_tag_stack: Vec<String>,
    #[allow(dead_code)]
    pub json_tag_stack: Vec<String>,
    pub var_list: Vec<String>,
    #[allow(dead_code)]
    pub code_list: Vec<String>,
    #[allow(dead_code)]
    pub vname_list: Vec<String>,
    pub flname_list: Vec<String>,
    pub flname_with_level_list: Vec<String>,
}

impl Flist2Code {
    /// Create new Flist2Code instance
    pub fn new() -> Flist2Code {
        Flist2Code {
            cur_flistp: "s_flistp".to_string(),
            prev_flistp: "s_flistp".to_string(),
            code: "".to_string(),
            vars: "".to_string(),
            c_code: "".to_string(),
            xml_code: "".to_string(),
            json_code: "".to_string(),
            tab: "".to_string(),
            cur_level: 0,

            cur_field: LField::new(),
            xml_tag_stack: Vec::new(),
            json_tag_stack: Vec::new(),
            var_list: Vec::new(),
            code_list: Vec::new(),
            vname_list: Vec::new(),
            flname_list: Vec::new(),
            flname_with_level_list: Vec::new(),
        }
    }

    /// Get variable name for flist datatype
    pub fn get_flist_var_name(&mut self, flname: &str) -> String {
        let var_name = flname.to_lowercase().replace("pin_fld_", "");
        let flist_var_name = var_name + "_flistp";
        return flist_var_name;
    }

    /// Get variable name for field
    pub fn get_var_name(&mut self, vname: &str, dtype: &str) -> String {
        let vname = vname.to_lowercase().replace("pin_fld_", "");
        return format!("{}_{}", dtype.to_lowercase(), vname);
    }

    /// Set current field by parsing line
    pub fn set_cur_field(&mut self, line: String) {
        self.cur_field.set_field(line);
    }

    /// Add variable for non-flist data types
    pub fn add_non_flist_variables(&mut self, field_name: String, field_value: String, field_type: String, field_level: &i8) -> String {
        let mut tab: String = String::from("\t\t");
        let mut ptr: String = String::from("*");
        let mut name = self.get_var_name(&field_name, &field_type);
        let mut value = String::from(field_value.to_string());
        name = format!("{}{}", name, field_level);

        let first_char = field_type.chars().next().map(|c| c.to_string()).unwrap_or_default();
        let dtype = BRM_DTYPE_DICT.get(&first_char).unwrap();

        match first_char.as_str() {
            "T" => { 
                ptr = "".to_string(); 
                tab = format!("\t{}", tab);
                value = format!("{}L", field_value.replace("(","").replace(")","").to_string());
            }
            "D" => { 
                value = format!("pbo_decimal_from_str(\"{}\", ebufp)", field_value.to_string());
            }
            "I"|"E" => { 
                ptr = "".to_string(); 
                tab = format!("\t{}", tab);
            },
            _ => { ptr = "".to_string(); },
        }

        if ! self.var_list.contains(&name) {
            self.vars.push_str(&format!("{}{}{}{}{} = {};\n", self.tab, dtype, tab, ptr, name, value));
            self.var_list.push(name.to_string());
        }
        else {
            if let Some(last) = name.chars().last().unwrap().to_digit(10) {
                name.truncate(name.len() - 1);
                name.push(std::char::from_digit(last + 1, 10).unwrap());
            }

            self.vars.push_str(&format!("{}{}{}{}{} = {};\n", self.tab, dtype, tab, ptr, name, value));
            self.var_list.push(name.to_string());
        }
        name
    }

    /// Add variable for flist data type
    pub fn add_flist_variable(&mut self, field_name: String, field_level: &i8) {
        let mut name = self.get_flist_var_name(&field_name);
        name = name.replace("flistp", &format!("{}_flistp", field_level));
    
        if ! self.flname_with_level_list.contains(&name) {
            self.vars.push_str(&format!("{}pin_flist_t\t\t*{} = NULL;\n", self.tab, name));
            self.flname_list.push(name.to_string());
            self.flname_with_level_list.push(name);
        }
    }

    /// Add C code for FLIST manipulation
    pub fn add_code(&mut self) {
        let field_level = self.cur_field.field_level;  
        let field_name = self.cur_field.field_name.to_string();  
        let field_type = self.cur_field.field_type.to_string();  
        let field_value = self.cur_field.field_value.to_string();  
        let mut field_arr_idx = self.cur_field.field_arr_idx.to_string();
        let mut pin_macro_name = "PIN_FLIST_FLD_SET";
        let mut value: String = String::new();

        match self.cur_field.field_type.as_str() {
            "TSTAMP" => { 
                value = format!("&{}",self.add_non_flist_variables(
                    field_name.to_string(), field_value.to_string(), field_type.to_string(), &field_level));
            },
            "INT"|"ENUM" => { 
                value = format!("&{}",self.add_non_flist_variables(
                    field_name.to_string(), field_value.to_string(), field_type.to_string(), &field_level));
            },
            "DECIMAL" => { 
                pin_macro_name = "PIN_FLIST_FLD_PUT";
                value = self.add_non_flist_variables(
                    field_name.to_string(), field_value.to_string(), field_type.to_string(), &field_level);
            },
            "ARRAY" | "SUBSTRUCT" => { 
                if self.cur_field.field_type.starts_with("ARRAY") {
                    pin_macro_name = "PIN_FLIST_ELEM_ADD";
                }		
                else {
                    pin_macro_name = "PIN_FLIST_SUBSTR_ADD";
                }
            
                let vname = self.get_var_name(&field_name, &field_type);
                if self.cur_level >= field_level {
                    let flname_list_length = self.flname_list.len();
                    let n = flname_list_length as i8 - field_level - 1;
                    for _ in 0..n {
                        self.prev_flistp = self.flname_list.pop().unwrap();
                    }
                    self.prev_flistp = self.flname_list.last().unwrap().to_string();
                    self.code.push_str("\n");
                    self.add_flist_variable(vname.to_string(), &field_level);
                }
                else {
                    self.add_flist_variable(vname.to_string(), &field_level);
                    self.prev_flistp = self.cur_flistp.to_string();
                }
                self.cur_flistp = self.get_flist_var_name(&vname.to_string());
                self.cur_flistp = self.cur_flistp.replace("flistp", &format!("{}_flistp", field_level));
                self.cur_level = field_level;
            },
            "POID" => { 
                pin_macro_name = "PIN_FLIST_FLD_SET";
                value = field_value.to_string();
                if !field_value.contains("NULL") {
                    pin_macro_name = "PIN_FLIST_FLD_PUT";
                    value = format!("PIN_POID_FROM_STR(\"{}\", NULL, ebufp)",field_value); 
                }
            }
            _ => { 
                value = field_value.to_string();
            }
        }

        if self.cur_field.field_type.starts_with("ARRAY") {
            if field_arr_idx == "*" {
                field_arr_idx = "PIN_ELEMID_ANY".to_string();
            }
            self.code.push_str(&format!("{}{} = {}({}, {}, {}, ebufp);\n", 
                self.tab, self.cur_flistp, pin_macro_name, self.prev_flistp, 
                field_name, field_arr_idx)); 
        }
        else if self.cur_field.field_type.starts_with("SUBSTRUCT") {
            self.code.push_str(&format!("{}{} = {}({}, {}, ebufp);\n",
                self.tab, self.cur_flistp, pin_macro_name,
                self.prev_flistp, field_name));
        }
        else {
            if self.cur_level > field_level {
                let flname_list_length = self.flname_list.len();
                let n = flname_list_length as i8 - field_level - 1;
                for _ in 0..n {
                    self.prev_flistp = self.flname_list.pop().unwrap();
                }
                self.prev_flistp = self.flname_list.last().unwrap().to_string();
                self.cur_flistp = self.flname_list.last().unwrap().to_string();
            }

            self.code.push_str(&format!("{}{}({}, {}, {}, ebufp);\n", 
                self.tab, pin_macro_name, self.cur_flistp, 
                field_name, value));
        }
    }

    /// Convert FLIST to C code
    pub fn convert_flist_2_ccode(&mut self, flist_str: String, flist_name: String) -> Result<String,String> {
        self.tab = if flist_name.starts_with("s_flistp") { "\t".to_string() } else { "".to_string() };
    
        self.code.clear();
    
        self.vars.push_str(&format!("{}pin_flist_t\t\t*{} = NULL;\n", self.tab, flist_name));
        self.code.push_str(&format!("{}{} = PIN_FLIST_CREATE(ebufp);\n",self.tab, flist_name));
        self.cur_flistp = flist_name.to_string();
        self.flname_list.push(flist_name.to_string());
        self.flname_with_level_list.push(format!("{}-0",flist_name.to_string()));
    
        for line in flist_str.lines() {
            self.set_cur_field(line.to_string());
            self.add_code();
        }
    
        self.c_code.push_str(&self.vars);
        self.c_code.push_str("\n");
        self.c_code.push_str(&self.code);
    
        Ok(self.c_code.to_string())
    }

    /// Add XML element for FLIST to XML conversion
    pub fn add_xml_element(&mut self) {
        let field_level = self.cur_field.field_level;
        let field_name = self.cur_field.field_name.to_string();
        let field_type = self.cur_field.field_type.to_string();
        let mut field_value = self.cur_field.field_value.to_string();
        let field_arr_idx = self.cur_field.field_arr_idx.to_string();

        match self.cur_field.field_type.as_str() {
            "ARRAY" | "SUBSTRUCT" => {
                if self.cur_level >= field_level && self.xml_tag_stack.len() > 1 {
                    let xml_tag_length = self.xml_tag_stack.len();
                    let n = xml_tag_length as i8 - field_level - 1;
                    for _ in 0..n {
                        let temp = self.xml_tag_stack.pop().unwrap();
                        self.tab.pop();
                        self.xml_code.push_str(&format!("{}</{}>\n", self.tab, temp));
                    }
                }

                if field_type.starts_with("ARRAY") {
                    self.xml_code.push_str(&format!("{}<{} elem=\"{}\">\n", self.tab, field_name, field_arr_idx))
                }
                else {
                    self.xml_code.push_str(&format!("{}<{}>\n", self.tab, field_name));
                }
                self.tab = format!("{}\t", self.tab);
                self.cur_level = field_level;
                self.xml_tag_stack.push(field_name);
            },
            _ => {
                if field_type.starts_with("STR") {
                    field_value = field_value.replace("\"","");
                }
                self.xml_code.push_str(&format!("{}<{}>{}</{}>\n", self.tab, field_name, field_value, field_name));
            }
        }
    }

    /// Convert FLIST to XML
    pub fn convert_flist_2_xml(&mut self, flist_str: String) -> Result<String, String> {
        self.xml_tag_stack.push("FLIST".to_string());
        self.xml_code.push_str("<flist>\n");
        self.tab = "\t".to_string();

        for line in flist_str.lines() {
            self.set_cur_field(line.to_string());
            self.add_xml_element();
        }
    
        let n = self.xml_tag_stack.len();
        if n > 1 {
            for _ in 0..n-1 {
                self.tab.pop();
                let temp = self.xml_tag_stack.pop().unwrap();		
                self.xml_code.push_str(&format!("{}</{}>\n", self.tab, temp));
            }
        }

        self.xml_code.push_str("</flist>\n");

        Ok(self.xml_code.to_string())
    }

    /// Convert FLIST to JSON (via XML transformation)
    pub fn convert_flist_2_json(&mut self, flist_str: String) -> Result<String, String> {
        let mut rng = rand::thread_rng();
        let input_file = format!("test_input_{}", rng.gen_range(0..1000));
        let output_file = format!("{}.out", input_file.to_string());

        let xml_flist_str = self.convert_flist_2_xml(flist_str.to_string()).unwrap();
        fs::write(input_file.to_string(), xml_flist_str.to_string()).expect("Failed to write file");

        Command::new("sh")
        .arg("-c")
        .arg(format!("cat {} | xml2json 2>/dev/null | jq 'walk(if type == \"object\" and has(\"$t\") then . |= .[\"$t\"] else . end)' | sed 's/\"elem\": \"\\([^\"]*\\)\",/\"_elem\": \"\\1\",/g' | tee {}", input_file, output_file))
        .output().unwrap();

        let result = fs::read_to_string(output_file.to_string()).unwrap_or_else(|_| {
            // Fallback if xml2json is not available
            "{\"error\": \"xml2json command not available\"}".to_string()
        });
    
        let _ = fs::remove_file(input_file);
        let _ = fs::remove_file(output_file);

        self.json_code.push_str(&result);

        Ok(self.json_code.to_string())
    }
}

/// Public function to convert FLIST to C code
pub fn convert_flist2code(content: String) -> String {
    let mut flist2code = Flist2Code::new();
    match flist2code.convert_flist_2_ccode(content.to_string(), "s_flistp".to_string()) {
        Ok(c_code) => c_code,
        Err(error) => format!("There is an error ({})", error),
    }
}

/// Public function to convert FLIST to XML
pub fn convert_flist2xml(content: String) -> String {
    let mut flist2code = Flist2Code::new();
    match flist2code.convert_flist_2_xml(content.to_string()) {
        Ok(xml) => xml,
        Err(error) => format!("There is an error ({})", error),
    }
}

/// Public function to convert FLIST to JSON
pub fn convert_flist2json(content: String) -> String {
    let mut flist2code = Flist2Code::new();
    match flist2code.convert_flist_2_json(content.to_string()) {
        Ok(json) => json,
        Err(error) => format!("There is an error ({})", error),
    }
}

/// Convert custom BRM class specification to PODL format
/// 
/// Input format: Lines with class definition and nested field structure
/// Output: Complete PODL class specification
pub fn convert_class_spec_to_podl(session_id: &str, data: String) -> String {
    let field_count = G_SESSION_MAP.lock().unwrap()
        .get(session_id)
        .map_or(0, |m| m.len());

    if field_count == 0 {
        return "Please load fields first !!!".to_string();
    }

    let content = data.replace("    ", "\t");
    let mut obj = ClassPODL::new();
    let mut cur_0th_level_field = "None".to_string();
    let mut cur_1st_level_field = "None".to_string();

    for line in content.lines() {
        if line.chars().all(|c| c.is_whitespace()) {
            continue;
        }

        let first_char = get_first_char(line);
        let parts: Vec<&str> = line.split('|').collect();
        
        if parts.len() < 2 {
            continue;
        }

        let second_parts: Vec<&str> = parts[1].split("->").collect();

        if first_char == '/' {
            // Class definition line
            let slash_count = line.chars().filter(|&ch| ch == '/').count();
            if slash_count > 1 {
                obj = ClassPODL::new_value(
                    parts[0].trim().to_string(),
                    second_parts[0].trim().to_string(),
                    "".to_string(),
                );
            } else if second_parts.len() > 1 {
                obj = ClassPODL::new_value(
                    parts[0].trim().to_string(),
                    second_parts[0].trim().to_string(),
                    second_parts[1].trim().to_string(),
                );
            }
        } else {
            // Field definition line
            let field = create_field_podl(session_id, parts[0], second_parts);
            let tab_count = get_begining_tab_count(line);
            
            match tab_count {
                1 => {
                    cur_0th_level_field = parts[0].trim().to_string();
                    obj.add_0th_lvl_field(cur_0th_level_field.clone(), field);
                }
                2 => {
                    cur_1st_level_field = parts[0].trim().to_string();
                    if cur_0th_level_field != "None" {
                        obj.add_1st_lvl_field(
                            cur_0th_level_field.clone(),
                            cur_1st_level_field.clone(),
                            field,
                        );
                    }
                }
                3 => {
                    let cur_2nd_level_field = parts[0].trim().to_string();
                    if cur_0th_level_field != "None" && cur_1st_level_field != "None" {
                        obj.add_2nd_lvl_field(
                            cur_0th_level_field.clone(),
                            cur_1st_level_field.clone(),
                            cur_2nd_level_field,
                            field,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    if obj.no_of_fields >= 0 {
        obj.generate_podl().join("\n")
    } else {
        "No data found".to_string()
    }
}