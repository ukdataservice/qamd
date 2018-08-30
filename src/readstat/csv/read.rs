
use readstat::bindings::*;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
// use std::path::Path;

use std::os::raw::{ c_int, c_void, c_char, c_long };
use std::ffi::{ CString, CStr };
// use std::ptr;

impl From<(usize, String)> for readstat_variable_t {
    fn from(index_variable: (usize, String)) -> Self {
        let (index, mut variable) = index_variable;

        let mut c_variable_name: [i8; 300usize] = [0i8; 300usize];
        as_fixed_size(&mut c_variable_name, &mut variable);

        readstat_variable_s {
            type_: readstat_type_t::READSTAT_TYPE_STRING,
            index: index as c_int, // ::std::os::raw::c_int,
            name: c_variable_name, //[::std::os::raw::c_char; 300usize],
            format: [0i8 as c_char; 256usize], // [::std::os::raw::c_char; 256usize],
            label: [0i8 as c_char; 1024usize], // [::std::os::raw::c_char; 1024usize],
            label_set: Box::into_raw(
                Box::new(
                    readstat_label_set_t {
                        type_: readstat_type_t::READSTAT_TYPE_STRING,
                        name: [0i8 as c_char; 256usize],
                        value_labels: Box::into_raw(
                            Box::new(
                                readstat_value_label_t {
                                    double_key: 0f64,
                                    int32_key: 0i32,
                                    tag: 0i8 as c_char,

                                    string_key: 0i8 as *mut c_char, // str_to_ptr!("".to_string()),
                                    string_key_len: 0usize,

                                    label: 0i8 as *mut c_char,
                                    label_len: 0usize,
                                })), // *mut readstat_value_label_t
                        value_labels_count: 0 as c_long,
                        value_labels_capacity: 0 as c_long,

                        variables: 0 as *mut c_void,
                        variables_count: 0 as c_long,
                        variables_capacity: 0 as c_long,
                    })),
                    offset: 0 as off_t,
                    storage_width: 0usize,
                    user_width: 0usize,
                    missingness: readstat_missingness_t {
                        missing_ranges: [readstat_value_t {
                            v: readstat_value_s__bindgen_ty_1 {
                                float_value: __BindgenUnionField::new(),
                                double_value: __BindgenUnionField::new(),
                                i8_value: __BindgenUnionField::new(),
                                i16_value: __BindgenUnionField::new(),
                                i32_value: __BindgenUnionField::new(),
                                string_value: __BindgenUnionField::new(),
                                bindgen_union_field: 0u64,
                            },
                            type_: readstat_type_t::READSTAT_TYPE_STRING,
                            tag: 0i8 as c_char,
                            _bitfield_1: 0u8,
                            __bindgen_padding_0: 0u16,
                        }; 32usize],
                        missing_ranges_count: 0 as c_long,
                    },
                    measure: readstat_measure_t::READSTAT_MEASURE_UNKNOWN,
                    alignment: readstat_alignment_t::READSTAT_ALIGNMENT_UNKNOWN,
                    display_width: 0 as c_int,
                    decimals: 0 as c_int,
                    skip: 0 as c_int,
                    index_after_skipping: 0 as c_int,
        }
    }
}

impl From<String> for readstat_value_t {
    fn from(_value: String)-> Self {
        //let c_str_value = str_to_ptr!(value);

        readstat_value_s {
            v: readstat_value_s__bindgen_ty_1 {
                float_value: __BindgenUnionField::new(),
                double_value: __BindgenUnionField::new(),
                i8_value: __BindgenUnionField::new(),
                i16_value: __BindgenUnionField::new(),
                i32_value: __BindgenUnionField::new(),
                string_value: __BindgenUnionField::new(), //__BindgenUnionField::as_ref(c_str_value),
                bindgen_union_field: 0u64,
            },
            // TODO: type detection
            type_: readstat_type_t::READSTAT_TYPE_STRING,
            tag: 0i8 as c_char,
            _bitfield_1: 0u8,
            __bindgen_padding_0: 0u16,
        }

        //value_r.v.string_value = c_str_value
    }
}

unsafe fn metadata_handler(handlers: &readstat_callbacks_t,
                           variables: &Vec<&str>,
                           records: &Vec<Vec<&str>>,
                           user_ctx: *mut c_void) -> i32 {
    if let Some(metadata_handler) = handlers.metadata {
        let metadata: *mut readstat_metadata_t = Box::into_raw(Box::new(readstat_metadata_t {
            row_count: records.len() as i64,
            var_count: variables.len() as i64,
            creation_time: 0,
            modified_time: 0,
            file_format_version: 0,
            compression: readstat_compress_t::READSTAT_COMPRESS_NONE,
            endianness: readstat_endian_t::READSTAT_ENDIAN_NONE,
            table_name: str_to_ptr!(""),
            file_label: str_to_ptr!(""),
            file_encoding: str_to_ptr!(""),
            _bitfield_1: 0u8,
            __bindgen_padding_0: [0, 0, 0, 0, 0, 0, 0]
        }));

        metadata_handler(metadata, user_ctx as *mut c_void)
    } else {
        return readstat_error_t::READSTAT_OK as i32;
    }
}

unsafe fn variable_handler(handlers: &readstat_callbacks_t,
                           index: usize,
                           variable: &str,
                           user_ctx: *mut c_void) -> i32 {
    if let Some(variable_handler) = handlers.variable {
        let variable_r: *mut readstat_variable_s =
            Box::into_raw(
                Box::new(
                    readstat_variable_s::from((index, variable.to_string()))));

        variable_handler(index as c_int,
                         variable_r,
                         0i8 as *const c_char,
                         user_ctx) as i32
    } else {
        return readstat_error_t::READSTAT_OK as i32;
    }
}

unsafe fn value_handler(handlers: &readstat_callbacks_t,
                        row_index: usize,
                        column_index: usize,
                        variable: &str,
                        value: &str,
                        user_ctx: *mut c_void) -> i32 {
    if let Some(value_handler) = handlers.value {
        let variable_r =
            Box::into_raw(
                Box::new(
                    readstat_variable_t::from((column_index,
                                               variable.to_string()))));

        let value_r = readstat_value_t::from(value.to_string());

        value_handler(row_index as c_int,
                      variable_r,
                      value_r,
                      user_ctx)
    } else {
        return readstat_error_t::READSTAT_OK as i32;
    }
}

pub unsafe extern "C" fn qamd_parse_csv(parser: *mut readstat_parser_t,
                                        path: *const c_char,
                                        user_ctx: *mut c_void) -> readstat_error_t {
    // open file
    // Err: READSTAT_ERROR_OPEN
    //
    // read file
    // Err: READSTAT_ERROR_READ
    //
    // build readstat_metadata_t & pass to handler.metadata
    // Err: if handler() != READSTAT_HANDLER_OK return READSTAT_ERROR_USER_ABORT
    //
    // read variables
    // read values
    // csv parser logic etc
    // call relevant handlers
    // (variable, value, value_label*, progress, note*, fweight?, error?)
    //
    // cleanup and return READSTAT_OK
    //
    // Don't touch the user_ctx other than to pass it through to handlers

    let parser_r = (*parser) as readstat_parser_t;
    let _parser_io: readstat_io_t = *parser_r.io;

    let path = ptr_to_str!(path);
    // debug!("path: {}", &path);
    // debug!("{:#?}", parser_io);
    // debug!("{:#?}", parser_r);
    // debug!("{:#?}", parser_io);

    match get_file_contents(&path) {
        Ok(contents) => {
            let lines = contents.split("\n").collect::<Vec<&str>>();
            let variables = lines.iter()
                .nth(0)
                .unwrap()
                .split(",").
                collect::<Vec<&str>>();

            let records = lines.iter()
                .skip(1)
                .map(|line| {
                    line.split(",").collect::<Vec<&str>>()
                }).collect::<Vec<Vec<&str>>>();

            // debug!("{:?}", &variables);
            // debug!("{:#?}", &records);

            let error = metadata_handler(&parser_r.handlers,
                                         &variables,
                                         &records,
                                         user_ctx);
            if error != readstat_error_t::READSTAT_OK as i32 {
                return readstat_error_t::READSTAT_ERROR_USER_ABORT;
            }

            for (index, variable) in variables.iter().enumerate() {

                let error = variable_handler(&parser_r.handlers,
                                             index,
                                             variable,
                                             user_ctx);

                if error != readstat_error_t::READSTAT_OK as i32 {
                    return readstat_error_t::READSTAT_ERROR_USER_ABORT;
                }


                // progress handler
            }

            // value handler
            // for (row_index, row) in records.iter().enumerate() {
            //     for (column_index, value) in row.iter().enumerate() {
            //         let error = value_handler(&parser_r.handlers,
            //                                   row_index,
            //                                   column_index,
            //                                   "variable here",
            //                                   value,
            //                                   user_ctx);

            //         if error != readstat_error_t::READSTAT_OK as i32 {
            //             return readstat_error_t::READSTAT_ERROR_USER_ABORT;
            //         }

            //         // progress handler
            //     }
            // }

            readstat_error_t::READSTAT_OK
        },
        Err(_) => return readstat_error_t::READSTAT_ERROR_OPEN,
    }
}

fn get_file_contents(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

fn as_fixed_size(a: &mut [i8], s: &mut String) {
    let bytes = unsafe { s.as_mut_vec() };

    for (first, second) in a.iter_mut().zip(bytes) {
        *first = *second as i8;
    }
}

