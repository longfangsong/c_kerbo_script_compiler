# CKerboScriptCompiler
This is a compiler which compiles C-like code into KerboScript, which is used by KSP mod [KOS](https://github.com/KSP-KOS/KOS).


with input:
```
var x = 0;
assign y = x;
for var i = 0;i<10;i=i+1 {
    x = i;
    print("y= ",y);
}
```
we get:
```KerboScript
DECLARE x TO 0.
LOCK y TO x.
FROM {DECLARE i TO 0.} UNTIL not(i<10) STEP {SET i to i+1.} DO {
    SET x to i.
    print "y= "+y.
}
```