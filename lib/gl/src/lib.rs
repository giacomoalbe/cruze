mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

use std::rc::Rc;
use std::ops::Deref;

use glutin::{Context, PossiblyCurrent};

#[derive(Clone)]
pub struct Gl {
    inner: Rc<bindings::Gl>,
}

impl Gl {
    pub fn load_with<F>(loadfn: F) -> Gl
        where F: FnMut(&'static str) -> *const types::GLvoid
    {
        Gl {
            inner: Rc::new(bindings::Gl::load_with(loadfn))
        }
    }

    pub fn load(context: &glutin::Context<PossiblyCurrent>) -> Gl {
        let gl = bindings::Gl::load_with(|ptr| {
            context.get_proc_address(ptr) as *const _
        });

        Gl {
            inner: Rc::new(gl)
        }
    }
}

impl Deref for Gl {
    type Target = bindings::Gl;

        fn deref(&self) -> &bindings::Gl {
            &self.inner
        }
}
