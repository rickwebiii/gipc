#[derive(PartialEq,Clone,Default)]
pub struct HelloRequest {
    // message fields
    pub name: ::std::string::String,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl HelloRequest {
    pub fn new() -> HelloRequest {
        ::std::default::Default::default()
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl ::std::fmt::Debug for HelloRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::Clear for HelloRequest {
    fn clear(&mut self) {
        self.clear_name();
        self.unknown_fields.clear();
    }
}

impl ::protobuf::Message for HelloRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> HelloRequest {
        HelloRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    |m: &HelloRequest| { &m.name },
                    |m: &mut HelloRequest| { &mut m.name },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HelloRequest>(
                    "HelloRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static HelloRequest {
        static mut instance: ::protobuf::lazy::Lazy<HelloRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HelloRequest,
        };
        unsafe {
            instance.get(HelloRequest::new)
        }
    }
}


static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0bhello.proto\x12\nhelloworld\"\"\n\x0cHelloRequest\x12\x12\n\x04nam\
    e\x18\x01\x20\x01(\tR\x04name\"&\n\nHelloReply\x12\x18\n\x07message\x18\
    \x01\x20\x01(\tR\x07message2I\n\x07Greeter\x12>\n\x08SayHello\x12\x18.he\
    lloworld.HelloRequest\x1a\x16.helloworld.HelloReply\"\0B6\n\x1bio.grpc.e\
    xamples.helloworldB\x0fHelloWorldProtoP\x01\xa2\x02\x03HLWJ\xcf\x08\n\
    \x06\x12\x04\x0f\0&\x01\n\xbf\x04\n\x01\x0c\x12\x03\x0f\0\x122\xb4\x04\
    \x20Copyright\x202015\x20gRPC\x20authors.\n\n\x20Licensed\x20under\x20th\
    e\x20Apache\x20License,\x20Version\x202.0\x20(the\x20\"License\");\n\x20\
    you\x20may\x20not\x20use\x20this\x20file\x20except\x20in\x20compliance\
    \x20with\x20the\x20License.\n\x20You\x20may\x20obtain\x20a\x20copy\x20of\
    \x20the\x20License\x20at\n\n\x20\x20\x20\x20\x20http://www.apache.org/li\
    censes/LICENSE-2.0\n\n\x20Unless\x20required\x20by\x20applicable\x20law\
    \x20or\x20agreed\x20to\x20in\x20writing,\x20software\n\x20distributed\
    \x20under\x20the\x20License\x20is\x20distributed\x20on\x20an\x20\"AS\x20\
    IS\"\x20BASIS,\n\x20WITHOUT\x20WARRANTIES\x20OR\x20CONDITIONS\x20OF\x20A\
    NY\x20KIND,\x20either\x20express\x20or\x20implied.\n\x20See\x20the\x20Li\
    cense\x20for\x20the\x20specific\x20language\x20governing\x20permissions\
    \x20and\n\x20limitations\x20under\x20the\x20License.\n\n\x08\n\x01\x08\
    \x12\x03\x11\0\"\n\t\n\x02\x08\n\x12\x03\x11\0\"\n\x08\n\x01\x08\x12\x03\
    \x12\04\n\t\n\x02\x08\x01\x12\x03\x12\04\n\x08\n\x01\x08\x12\x03\x13\00\
    \n\t\n\x02\x08\x08\x12\x03\x13\00\n\x08\n\x01\x08\x12\x03\x14\0!\n\t\n\
    \x02\x08$\x12\x03\x14\0!\n\x08\n\x01\x02\x12\x03\x16\x08\x12\n.\n\x02\
    \x06\0\x12\x04\x19\0\x1c\x01\x1a\"\x20The\x20greeting\x20service\x20defi\
    nition.\n\n\n\n\x03\x06\0\x01\x12\x03\x19\x08\x0f\n\x1f\n\x04\x06\0\x02\
    \0\x12\x03\x1b\x025\x1a\x12\x20Sends\x20a\x20greeting\n\n\x0c\n\x05\x06\
    \0\x02\0\x01\x12\x03\x1b\x06\x0e\n\x0c\n\x05\x06\0\x02\0\x02\x12\x03\x1b\
    \x10\x1c\n\x0c\n\x05\x06\0\x02\0\x03\x12\x03\x1b'1\n=\n\x02\x04\0\x12\
    \x04\x1f\0!\x01\x1a1\x20The\x20request\x20message\x20containing\x20the\
    \x20user's\x20name.\n\n\n\n\x03\x04\0\x01\x12\x03\x1f\x08\x14\n\x0b\n\
    \x04\x04\0\x02\0\x12\x03\x20\x02\x12\n\r\n\x05\x04\0\x02\0\x04\x12\x04\
    \x20\x02\x1f\x16\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03\x20\x02\x08\n\x0c\n\
    \x05\x04\0\x02\0\x01\x12\x03\x20\t\r\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\
    \x20\x10\x11\n;\n\x02\x04\x01\x12\x04$\0&\x01\x1a/\x20The\x20response\
    \x20message\x20containing\x20the\x20greetings\n\n\n\n\x03\x04\x01\x01\
    \x12\x03$\x08\x12\n\x0b\n\x04\x04\x01\x02\0\x12\x03%\x02\x15\n\r\n\x05\
    \x04\x01\x02\0\x04\x12\x04%\x02$\x14\n\x0c\n\x05\x04\x01\x02\0\x05\x12\
    \x03%\x02\x08\n\x0c\n\x05\x04\x01\x02\0\x01\x12\x03%\t\x10\n\x0c\n\x05\
    \x04\x01\x02\0\x03\x12\x03%\x13\x14b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
