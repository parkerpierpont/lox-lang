use std::{rc::Rc, sync::RwLock};

use downcast::{downcast, Any};

pub trait LoxObjectBase: Any + PrimitiveLoxObject {}
pub trait PrimitiveLoxObject {
    fn instance_name(&self) -> &'static str;
}

downcast!(dyn LoxObjectBase);

#[derive(Clone)]
pub struct LoxObject(Rc<RwLock<dyn LoxObjectBase>>);

impl LoxObject {
    pub fn instance_name(&self) -> &'static str {
        if let Ok(v) = self.0.try_read() {
            v.instance_name()
        } else {
            "Nil"
        }
    }

    pub fn get_boolean(&self) -> bool {
        if let Ok(val) = self.0.try_read() {
            if let Ok(r) = val.downcast_ref::<LoxBoolean>() {
                return r.0;
            }
        }
        false
    }

    pub fn get_number(&self) -> f64 {
        if let Ok(val) = self.0.try_read() {
            if let Ok(r) = val.downcast_ref::<LoxNumber>() {
                return r.0;
            }
        }
        0.0
    }

    pub fn get_string(&self) -> String {
        if let Ok(val) = self.0.try_read() {
            if let Ok(r) = val.downcast_ref::<LoxString>() {
                return r.0.clone();
            }
        }
        "".to_string()
    }

    pub fn is_truthy(&self) -> bool {
        match self.instance_name() {
            "Nil" => false,
            "Boolean" => self.get_boolean(),
            _ => true,
        }
    }

    pub fn stringify(&self) -> String {
        match self.instance_name() {
            "Nil" => "nil".to_string(),
            "Number" => format!("{:.2}", self.get_number()),
            "String" => self.get_string(),
            "Boolean" => (if self.get_boolean() { "true" } else { "false" }).to_string(),
            _ => unreachable!(),
        }
    }
}

impl PartialEq for LoxObject {
    fn eq(&self, other: &Self) -> bool {
        let self_ty = self.instance_name();
        let other_ty = other.instance_name();
        match (self_ty, other_ty) {
            ("Nil", "Nil") => true,
            ("Nil", _) => false,
            ("Number", "Number") => self.get_number() == other.get_number(),
            ("String", "String") => self.get_string() == other.get_string(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LoxBoolean(pub bool);
impl LoxBoolean {
    pub fn new(value: bool) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxBoolean(value))))
    }
}
impl LoxObjectBase for LoxBoolean {}
impl PrimitiveLoxObject for LoxBoolean {
    fn instance_name(&self) -> &'static str {
        "Boolean"
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LoxNumber(pub f64);
impl LoxNumber {
    pub fn new(value: f64) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxNumber(value))))
    }
}
impl LoxObjectBase for LoxNumber {}
impl PrimitiveLoxObject for LoxNumber {
    fn instance_name(&self) -> &'static str {
        "Number"
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LoxString(pub String);
impl LoxString {
    pub fn new(value: String) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxString(value))))
    }
}
impl LoxObjectBase for LoxString {}
impl PrimitiveLoxObject for LoxString {
    fn instance_name(&self) -> &'static str {
        "String"
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LoxNil;
impl LoxNil {
    pub fn new() -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxNil)))
    }
}
impl LoxObjectBase for LoxNil {}
impl PrimitiveLoxObject for LoxNil {
    fn instance_name(&self) -> &'static str {
        "Nil"
    }
}
