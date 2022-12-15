use std::ffi::{CString, c_void};

use vips_sys as s;

use crate::{
    VipsImage,
    VipsError
};

pub trait ToGValue {
    fn to_GValue(&self) -> Option<s::GValue>;
}

impl<T> ToGValue for Option<T> where T: ToGValue {
    fn to_GValue(&self) -> Option<s::GValue> {
        match self {
            Some(v) => v.to_GValue(),
            None => None
        }
    }
}

impl ToGValue for i32 {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::define_G_TYPE_INT
            );
    
            s::g_value_set_int(
                &mut g_value,
                *self
            );
    
            return Some(g_value);
        }
    }
}

impl ToGValue for u32 {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::define_G_TYPE_UINT
            );
    
            s::g_value_set_uint(
                &mut g_value,
                *self
            );
    
            return Some(g_value);
        }
    }
}

impl ToGValue for f32 {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::define_G_TYPE_FLOAT
            );
    
            s::g_value_set_float(
                &mut g_value,
                *self
            );
    
            return Some(g_value);
        }
    }
}

impl ToGValue for f64 {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::define_G_TYPE_DOUBLE
            );
    
            s::g_value_set_double(
                &mut g_value,
                *self
            );
    
            return Some(g_value);
        }
    }
}

impl ToGValue for bool {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::define_G_TYPE_BOOLEAN
            );
    
            s::g_value_set_boolean(
                &mut g_value,
                *self as i32
            );
    
            return Some(g_value);
        }
    }
}

impl ToGValue for VipsImage {
    fn to_GValue(&self) -> Option<s::GValue> {
        unsafe {
            let mut g_value: s::GValue = std::mem::zeroed();
    
            s::g_value_init(
                &mut g_value,
                s::vips_image_get_type()
            );
    
            s::g_value_set_object(
                &mut g_value,
                self.ptr as *mut c_void
            );
    
            return Some(g_value);
        }
    }
}

macro_rules! struct_to_arg_init {
    () => {
        // init other args
        $(
            let g_value = args.$param_name.to_GValue();
            match g_value {
                Some(v) => {
                    let prop_name_c_str = CString::new(stringify!($param_name))?;
                    s::g_object_set_property(
                        op as *mut s::GObject,
                        prop_name_c_str.as_ptr(),
                        &v
                    );
                },
                _ => {} // if it's not Some(v), this arg wasn't there
            };
        )*
    }
}

/// Define an operator like:
/// 
/// ```ignore
/// define_operator!(conv, struct ConvArgs<'a> {
///     mask: &'a VipsImage,
///     precision: Option<vips_sys::VipsPrecision>,
///     layers: Option<u32>,
///     cluster: Option<u32>
/// });
/// ```
/// 
/// If you need to specify a custom input parameter name, you can pass a string
/// literal as the second argument.
/// 
/// ```ignore
/// define_operator!(add, "left", struct AddArgs<'a> {
///     right: &'a VipsImage
/// });
/// ```
/// 
/// The name of the argument struct will be discarded and the resulting struct
/// will always have the name `OpArgs`. The resulting operator can then be
/// called this way:
/// 
/// ```ignore
/// let convolved = input_img.conv(conv::OpArgs{ ... });
/// ```
#[macro_export]
macro_rules! define_operator {
    (
        $op_name:ident,
        $input_name:literal,
        $(#[$meta:meta])*
        $struct_vis:vis struct $param_struct_name:ident $(<$lt:lifetime>)? {
            $(
                $(#[$param_meta:meta])*
                $param_vis:vis $param_name:ident: $param_type:ty
            ),*
        }
    ) => {
        mod $op_name {
            use super::*;

            use std::ffi::{CString, c_void};
            use vips_sys as s;
            use crate::*;

            $(#[$meta])*
            pub struct OpArgs $(<$lt>)? {
                $(
                    $(#[$param_meta:meta])*
                    $param_vis $param_name : $param_type
                ),*
            }

            impl VipsImage {
                pub fn $op_name(&self, args: OpArgs) -> Result<VipsImage, VipsError> {
                    unsafe {
                        let op_name_c_str = CString::new(stringify!($op_name))?;
                        let op = s::vips_operation_new(op_name_c_str.as_ptr());

                        if op == std::ptr::null_mut() {
                            return Err(VipsError::new("Could not create operation"));
                        }

                        let g_value = match self.to_GValue() {
                            Some(value) => value,
                            None => return Err(VipsError::new("Failed to convert input image to GValue"))
                        };

                        let prop_name_c_str = CString::new($input_name)?;
                        s::g_object_set_property(
                            op as *mut s::GObject,
                            prop_name_c_str.as_ptr(),
                            &g_value
                        );
                        
                        // init other args
                        $(
                            let g_value = args.$param_name.to_GValue();
                            match g_value {
                                Some(v) => {
                                    let prop_name_c_str = CString::new(stringify!($param_name))?;
                                    s::g_object_set_property(
                                        op as *mut s::GObject,
                                        prop_name_c_str.as_ptr(),
                                        &v
                                    );
                                },
                                _ => {} // if it's not Some(v), this arg wasn't there
                            };
                        )*

                        // run the op
                        let cached_op = s::vips_cache_operation_build(op);
                        s::g_object_unref(op as *mut c_void);

                        if cached_op == std::ptr::null_mut() {
                            // TODO: add custom message for context
                            return Err(VipsError::new_from_vips_state());
                        }

                        // get result
                        let mut g_value: s::GValue = std::mem::zeroed();
                        s::g_value_init(
                            &mut g_value,
                            s::vips_image_get_type()
                        );
                        let prop_name_c_str = CString::new("out")?;
                        s::g_object_get_property(
                            cached_op as *mut s::GObject,
                            prop_name_c_str.as_ptr(),
                            &mut g_value
                        );
                        let out = VipsImage::from_c_ptr(
                            s::g_value_get_object(
                                &g_value
                            ) as *mut s::VipsImage
                        )?;
                        // g_value_get_object() does not ref the object, so we need to make
                        // a ref for out to hold.
                        s::g_object_ref(out.ptr as *mut c_void);
                        s::g_value_unset(&mut g_value);

                        s::vips_object_unref_outputs(cached_op as *mut s::VipsObject);
                        s::g_object_unref(cached_op as *mut c_void);

                        return Ok(out);
                    }
                }
            }
        }
    };
    ( $op_name:ident, $($param_struct_def:tt)* ) => {
        define_operator!($op_name, "in", $($param_struct_def)*);
    };
}


// === ARITHMETIC OPERATORS ===

define_operator!(add, "left", struct Args<'a> {
    pub right: &'a VipsImage
});

define_operator!(subtract, "left", struct Args<'a> {
    pub right: &'a VipsImage
});

// TODO: other ops, operator overloading, arithmetic with constants

// === colour ===
// === conversion ===

// === CONVOLUTION OPERATORS ===

define_operator!(conv, struct Args<'a> {
    pub mask: &'a VipsImage,
    pub precision: Option<vips_sys::VipsPrecision>,
    pub layers: Option<u32>,
    pub cluster: Option<u32>
});

// === VipsForeign ===
// === freqfilt ===
// === histogram ===
// === draw ===
// === VipsInterpolate ===
// === morphology ===
// === mosaicing ===
// === create ===
// === resample ===

#[cfg(test)]
mod operation_tests {
    use super::*;
    use crate::ensure_vips_init_or_exit;
    use std::path::PathBuf;

    #[test]
    fn convolve() {
        ensure_vips_init_or_exit();

        let img = VipsImage::new_from_file(PathBuf::from("./data/test.jpg"))
            .expect("Image could not be created from file");
        assert_ne!(img.ptr, std::ptr::null_mut());

        let kernel_size = 64;
        let kernel = VipsImage::new_matrix(kernel_size, kernel_size)
            .expect("Could not create kernel");

        let kernel_n_px: f64 = (kernel_size * kernel_size).try_into().unwrap();
        let kernel_v = 16f64 / kernel_n_px; // TODO: figure out why the kernel needs sum ~16
        unsafe {
            vips_sys::vips_draw_rect1(kernel.ptr,
                kernel_v,
                0, 0, kernel_size, kernel_size, 0);
        }
        let convolved = img.conv(conv::OpArgs{
            mask: &kernel,
            precision: None, layers: None, cluster: None
        })
            .expect("Error while applying operation `conv`");

        convolved.write_to_file(PathBuf::from("./data/test_convolved.jpg"))
            .expect("Could not save image to file");
    }

    #[test]
    fn add() {
        ensure_vips_init_or_exit();

        let img = VipsImage::new_from_file(PathBuf::from("./data/test.jpg"))
            .expect("Image could not be created from file");
        assert_ne!(img.ptr, std::ptr::null_mut());

        let _added = img.add(add::OpArgs{right: &img})
            .expect("Could not add image");

        // TODO: check if pixel values make sense
    }

    #[test]
    fn subtract() {
        ensure_vips_init_or_exit();

        let img = VipsImage::new_from_file(PathBuf::from("./data/test.jpg"))
            .expect("Image could not be created from file");
        assert_ne!(img.ptr, std::ptr::null_mut());

        let _subtracted = img.subtract(subtract::OpArgs{right: &img})
            .expect("Could not subtract image");

        // TODO: check if pixel values make sense
    }
}
