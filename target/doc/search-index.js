var N=null,E="",T="t",U="u",searchIndex={};
var R=["unsize","valuea","result","option","try_from","try_into","borrow","borrow_mut","type_id","typeid"];

searchIndex["stack_dst"]={"doc":"Support for storing dynamically-sized types on the stack","i":[[3,"ValueA","stack_dst","Stack-allocated dynamically sized type",N,N],[3,"StackA",E,"A fixed-capacity stack that can contain dynamically-sized…",N,N],[11,"new",E,"Construct a stack-based DST",0,[[[R[0]]],[[R[1]],[R[0]],[R[2],[R[1]]]]]],[11,"new",E,"Construct a new (empty) stack",1,[[],["stacka"]]],[11,"is_empty",E,"Tests if the stack is empty",1,[[["self"]],["bool"]]],[11,"push",E,"Push a value at the top of the stack",1,[[["self"],[R[0]]],[[R[0]],[R[2]]]]],[11,"top",E,"Returns a pointer to the top item on the stack",1,[[["self"]],[[R[3]],[T]]]],[11,"top_mut",E,"Returns a pointer to the top item on the stack…",1,[[["self"]],[[T],[R[3]]]]],[11,"pop",E,"Pop the top item off the stack",1,[[["self"]]]],[11,"push_str",E,"Push the contents of a string slice as an item onto the…",1,[[["str"],["self"]],[R[2]]]],[11,"push_cloned",E,"Pushes a set of items (cloning out of the input slice)",1,[[["self"]],[R[2]]]],[6,"Value",E,"Stack-allocated DST (using a default size)",N,N],[8,"DataBuf",E,"Trait used to represent the data buffer for StackDSTA.",N,N],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[4],E,E,0,[[[U]],[R[2]]]],[11,R[5],E,E,0,[[],[R[2]]]],[11,R[6],E,E,0,[[["self"]],[T]]],[11,R[7],E,E,0,[[["self"]],[T]]],[11,R[8],E,E,0,[[["self"]],[R[9]]]],[11,"into",E,E,1,[[],[U]]],[11,"from",E,E,1,[[[T]],[T]]],[11,R[4],E,E,1,[[[U]],[R[2]]]],[11,R[5],E,E,1,[[],[R[2]]]],[11,R[6],E,E,1,[[["self"]],[T]]],[11,R[7],E,E,1,[[["self"]],[T]]],[11,R[8],E,E,1,[[["self"]],[R[9]]]],[11,"drop",E,E,0,[[["self"]]]],[11,"drop",E,E,1,[[["self"]]]],[11,"default",E,E,1,[[],["self"]]],[11,"deref",E,E,0,[[["self"]],[T]]],[11,"deref_mut",E,E,0,[[["self"]],[T]]]],"p":[[3,"ValueA"],[3,"StackA"]]};
initSearch(searchIndex);addSearchOptions(searchIndex);