var searchIndex = JSON.parse('{\
"stack_dst":{"doc":"Support for storing dynamically-sized types on the stack","i":[[8,"DataBuf","stack_dst","Trait used to represent a data buffer, typically you’ll …",null,null],[3,"StackA","","A fixed-capacity stack that can contain dynamically-sized …",null,null],[6,"Value","","Stack-allocated DST (using a default size)",null,null],[3,"ValueA","","Stack-allocated dynamically sized type",null,null],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","","",1,[[]]],[11,"into","","",1,[[]]],[11,"to_string","","",1,[[],["string",3]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"into_future","","",1,[[]]],[11,"drop","","",0,[[]]],[11,"drop","","",1,[[]]],[11,"default","","",0,[[]]],[11,"deref","","",1,[[]]],[11,"deref_mut","","",1,[[]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"poll","","",1,[[["context",3],["pin",3]],["poll",4]]],[11,"new","","Construct a new (empty) stack",0,[[],["stacka",3]]],[11,"is_empty","","Tests if the stack is empty",0,[[],["bool",15]]],[11,"push_stable","","Push a value at the top of the stack (without using <code>Unsize</code>…",0,[[["fnonce",8]],["result",4]]],[11,"top","","Returns a pointer to the top item on the stack",0,[[],["option",4]]],[11,"top_mut","","Returns a pointer to the top item on the stack …",0,[[],["option",4]]],[11,"pop","","Pop the top item off the stack",0,[[]]],[11,"push_str","","Push the contents of a string slice as an item onto the …",0,[[["str",15]],["result",4]]],[11,"push_cloned","","Pushes a set of items (cloning out of the input slice)",0,[[],["result",4]]],[11,"new_stable","","Construct a stack-based DST (without needing <code>Unsize</code>)",1,[[["fnonce",8]],[["result",4],["valuea",3]]]],[11,"new_raw","","UNSAFE: <code>data</code> must point to <code>size</code> bytes, which shouldn’t …",1,[[["usize",15]],[["valuea",3],["option",4]]]]],"p":[[3,"StackA"],[3,"ValueA"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);