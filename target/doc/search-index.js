var searchIndex={};
searchIndex["stack_dst"] = {"doc":"Support for storing dynamically-sized types on the stack","i":[[3,"StackA","stack_dst","A fixed-capacity stack that can contain dynamically-sized…",null,null],[3,"ValueA","","Stack-allocated dynamically sized type",null,null],[11,"new","","Construct a new (empty) stack",0,[[],["stacka"]]],[11,"is_empty","","Tests if the stack is empty",0,[[["self"]],["bool"]]],[11,"push_stable","","Push a value at the top of the stack (without using…",0,[[["u"],["self"],["fnonce"]],["result"]]],[11,"top","","Returns a pointer to the top item on the stack",0,[[["self"]],[["t"],["option"]]]],[11,"top_mut","","Returns a pointer to the top item on the stack…",0,[[["self"]],[["option"],["t"]]]],[11,"pop","","Pop the top item off the stack",0,[[["self"]]]],[11,"push_str","","Push the contents of a string slice as an item onto the…",0,[[["str"],["self"]],["result"]]],[11,"push_cloned","","Pushes a set of items (cloning out of the input slice)",0,[[["self"]],["result"]]],[11,"new_stable","","Construct a stack-based DST (without needing `Unsize`)",1,[[["u"],["fnonce"]],[["valuea"],["result",["valuea"]]]]],[11,"new_raw","","UNSAFE: `data` must point to `size` bytes, which shouldn't…",1,[[["usize"]],[["option",["valuea"]],["valuea"]]]],[6,"Value","","Stack-allocated DST (using a default size)",null,null],[8,"DataBuf","","Trait used to represent a data buffer, typically you'll…",null,null],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"try_into","","",0,[[],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"type_id","","",0,[[["self"]],["typeid"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[],["u"]]],[11,"to_string","","",1,[[["self"]],["string"]]],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"try_into","","",1,[[],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"type_id","","",1,[[["self"]],["typeid"]]],[11,"drop","","",0,[[["self"]]]],[11,"drop","","",1,[[["self"]]]],[11,"default","","",0,[[],["self"]]],[11,"deref","","",1,[[["self"]],["t"]]],[11,"deref_mut","","",1,[[["self"]],["t"]]],[11,"fmt","","",1,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",1,[[["self"],["formatter"]],["result"]]],[11,"poll","","",1,[[["self"],["context"],["pin"]],["poll"]]]],"p":[[3,"StackA"],[3,"ValueA"]]};
addSearchOptions(searchIndex);initSearch(searchIndex);