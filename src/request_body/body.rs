use oas3::{
    spec::{FromRef, ObjectOrReference, RefError},
    Schema, Spec,
};


pub fn parse_schema(
    schema: Option<ObjectOrReference<Schema>>,
    spec: &Spec,
) -> Result<Option<Schema>, RefError> {
    match schema {
        Some(s) => match s {
            ObjectOrReference::Object(s) => Ok(Some(s)),
            ObjectOrReference::Ref { ref_path } => match Schema::from_ref(&spec, &ref_path) {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e),
            },
        },

        None => Ok(None),
    }
}

