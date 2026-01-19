
## CreateSession
```
<method name="CreateSession">
      <annotation name="org.qtproject.QtDBus.QtTypeName.In0" value="QVariantMap"/>
      <arg type="a{sv}" name="options" direction="in"/>
      <arg type="o" name="handle" direction="out"/>
    </method>
```




## Start
```
 <method name="Start">
      <annotation name="org.qtproject.QtDBus.QtTypeName.In2" value="QVariantMap"/>
      <arg type="o" name="session_handle" direction="in"/>
      <arg type="s" name="parent_window" direction="in"/>
      <arg type="a{sv}" name="options" direction="in"/>
      <arg type="o" name="handle" direction="out"/>
    </method>
```
## Common DBus types and their rust equivalents
- `u` - int (u32)
- `o` - (Owned)ObjectPath - looks like `/org/freedesktop/portal/desktop`
- `a{}` - hashmap
- `s` - string
- `v` - variant - zvariant::Value 
- `a{sv}` - hashmap with string as key and variant as value: `HashMap::<&str, Value>`