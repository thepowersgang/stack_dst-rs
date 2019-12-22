var searchIndex={};
searchIndex["stack_dst"] = {"doc":"Support for storing dynamically-sized types on the stack","i":[[3,"ValueA","stack_dst","Stack-allocated dynamically sized type",null,null],[3,"StackA","","A fixed-capacity stack that can contain dynamically-sized…",null,null],[11,"new","","Construct a stack-based DST",0,[[["unsize"]],[["unsize"],["result",["valuea"]],["valuea"]]]],[11,"new","","Construct a new (empty) stack",1,[[],["stacka"]]],[11,"is_empty","","Tests if the stack is empty",1,[[["self"]],["bool"]]],[11,"push","","Push a value at the top of the stack",1,[[["self"],["unsize"]],[["result"],["unsize"]]]],[11,"top","","Returns a pointer to the top item on the stack",1,[[["self"]],[["option"],["t"]]]],[11,"top_mut","","Returns a pointer to the top item on the stack…",1,[[["self"]],[["option"],["t"]]]],[11,"pop","","Pop the top item off the stack",1,[[["self"]]]],[11,"push_str","","Push the contents of a string slice as an item onto the…",1,[[["str"],["self"]],["result"]]],[11,"push_cloned","","Pushes a set of items (cloning out of the input slice)",1,[[["self"]],["result"]]],[6,"Value","","Stack-allocated DST (using a default size)",null,null],[8,"DataBuf","","Trait used to represent a data buffer, typically you'll…",null,null],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"try_into","","",0,[[],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"type_id","","",0,[[["self"]],["typeid"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[],["u"]]],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"try_into","","",1,[[],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"type_id","","",1,[[["self"]],["typeid"]]],[11,"drop","","",0,[[["self"]]]],[11,"drop","","",1,[[["self"]]]],[11,"default","","",1,[[],["self"]]],[11,"deref","","",0,[[["self"]],["t"]]],[11,"deref_mut","","",0,[[["self"]],["t"]]]],"p":[[3,"ValueA"],[3,"StackA"]]};
addSearchOptions(searchIndex);initSearch(searchIndex);