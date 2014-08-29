var searchIndex = {};
searchIndex['small_step_simple'] = {"items":[[0,"","small_step_simple","This is an implementation of the small-step approach to the SIMPLE language as introduced by\n[Tom Stuart](https://twitter.com/tomstuart) in \"Understanding Computation\", Chapter 1, \"The Meaning of Programs\".\nSee his website: <http://computationbook.com/>."],[1,"Machine","","Our virtual machine, executing our constructed AST step-by-step"],[2,"Element","","Our AST elements."],[12,"Number","","A simple number object, this cannot be reduced further.",0],[12,"Add","","An addition of two elements.",0],[12,"Multiply","","A multiplication of two elements.",0],[12,"Boolean","","A simple boolean object, this cannot be reduced further.",0],[12,"LessThan","","A less-than relation check of two elements. Elements should reduce to a number to be\ncomparable.",0],[12,"Variable","","A variable, will be replaced by its value when reducing.",0],[12,"Assign","","A variable assignment. Only completely reduced values are assigned. No type checks.",0],[12,"Sequence","","A sequence of two elements. The first element is reduced completely before the second is\ntouched.",0],[12,"IfElse","","A if-else block. Condition needs to reduce to a Boolean. No type checking.\nIf `condition` reduces to true, the `consequence` is used furhter, otherwise the `alternative`",0],[12,"While","","A while loop. Runs until the `condition` reduces to false.",0],[12,"DoNothing","","A simple no-op statement.",0],[10,"eq","","",0],[10,"ne","","",0],[10,"clone","","",0],[10,"fmt","","Output a user-readable representation of the expression",0],[10,"is_reducible","","Wether or not an expression is reducible. See Element for more info.",0],[10,"value","","Get the actual value of a Number.\nFails for other elements than Number and Boolean.\nBoolean maps to Integers: true=1, false=0.",0],[10,"reduce","","Reduce the expression according to the rules for the current element.",0],[10,"new","","Create a new machine with a given expression and an environment",1],[10,"new_with_empty_env","","Create a new machine with a given expression and an _empty_ environment",1],[10,"clone_env","","As the environment is passed in immutable, we need to clone it to get it back",1],[10,"step","","Reduce one step of our current expression",1],[10,"run","","Reduce until we reached a non-reducible expression.\nThis prints the current expression before each step.",1]],"paths":[[2,"Element"],[1,"Machine"]]};
initSearch(searchIndex);
