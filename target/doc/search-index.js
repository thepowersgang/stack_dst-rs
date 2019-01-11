var N = null;var searchIndex = {};
searchIndex["stack_dst"]={"doc":"Support for storing dynamically-sized types on the stack","items":[[3,"ValueA","stack_dst","Stack-allocated dynamically sized type",N,N],[3,"StackA","","A fixed-capacity stack that can contain dynamically-sized…",N,N],[11,"new","","Construct a stack-based DST",0,[[["u"]],["result",["valuea"]]]],[11,"new","","Construct a new (empty) stack",1,[[],["stacka"]]],[11,"is_empty","","Tests if the stack is empty",1,[[["self"]],["bool"]]],[11,"push","","Push a value at the top of the stack",1,[[["self"],["u"]],["result"]]],[11,"top","","Returns a pointer to the top item on the stack",1,[[["self"]],["option"]]],[11,"top_mut","","Returns a pointer to the top item on the stack…",1,[[["self"]],["option"]]],[11,"pop","","Pop the top item off the stack",1,[[["self"]]]],[11,"push_str","","Push the contents of a string slice as an item onto the…",1,[[["self"],["str"]],["result"]]],[11,"push_cloned","","Pushes a set of items (cloning out of the input slice)",1,N],[6,"Value","","Stack-allocated DST (using a default size)",N,N],[8,"DataBuf","","Trait used to represent the data buffer for StackDSTA.",N,N],[11,"from","","",0,[[["t"]],["t"]]],[11,"into","","",0,[[["self"]],["u"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"try_into","","",0,[[["self"]],["result"]]],[11,"get_type_id","","",0,[[["self"]],["typeid"]]],[11,"from","","",1,[[["t"]],["t"]]],[11,"into","","",1,[[["self"]],["u"]]],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"try_into","","",1,[[["self"]],["result"]]],[11,"get_type_id","","",1,[[["self"]],["typeid"]]],[11,"drop","","",0,[[["self"]]]],[11,"drop","","",1,[[["self"]]]],[11,"default","","",1,[[],["self"]]],[11,"deref_mut","","",0,[[["self"]],["t"]]],[11,"deref","","",0,[[["self"]],["t"]]]],"paths":[[3,"ValueA"],[3,"StackA"]]};
initSearch(searchIndex);addSearchOptions(searchIndex);
