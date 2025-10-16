### Keys

```yaml
a: 123
b: 456
c: 789
```

```
...
```

### Nesting

```yaml
# a:
# 	b: 
# 		c: 123
person:
  name: John Doe
  age: 30
  address:
    street: 123 Main St
    city: Example City

```

```
...
```

### Comments

```yaml
hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In
```

```
...
```

### Lists

```yaml
- Mark McGwire
- Sammy Sosa
- Ken Griffey
```

```
...
```

### Lists

```yaml
objects:
- a
- b
- c
```

```
bad
>[Slice("a"), Slice("b")]
> -> String("c: 123")
```

### Lists of objects

```yaml
objects:
	- name: a
		data: 5
	- name: b
	- name: c
		data: 7
```

```
bad
>[Slice("objects"), Index { index: 0, bracketed: false }, Slice("name")]
> -> String("a")
>[Slice("objects"), Index { index: 0, bracketed: false }, Slice("data")]
> -> Number("5")
>[Slice("objects"), Index { index: 0, bracketed: false }, Index { index: 1, bracketed: false }, Slice("name")]
> -> String("b")
>[Slice("objects"), Index { index: 0, bracketed: false }, Index { index: 2, bracketed: false }, Slice("name")]
> -> String("c")
>[Slice("objects"), Index { index: 0, bracketed: false }, Index { index: 2, bracketed: false }, Slice("data")]
> -> Number("7")
```

### Lists of objects (syntax two)

```yaml
-
  name: Mark McGwire
  hr:   65
  avg:  0.278
-
  name: Sammy Sosa
  hr:   63
  avg:  0.288
```

```
...
```

### Multiline strings

```yaml
objects: |
	this is a paragraph
	that spans multiple lines
```

```
...
```

## Expression syntax

### Array literal

```yaml
- [name        , hr, avg  ]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa  , 63, 0.288]
```

```
...
```

### Object literal

```yaml
Mark McGwire: {hr: 65, avg: 0.278}
Sammy Sosa: {
    hr: 63,
    avg: 0.288,
 }
```

```
...
```

## Multiple documents

### Multiple documents 1

```yaml
# Ranking of 1998 home runs
---
- Mark McGwire
- Sammy Sosa
- Ken Griffey

# Team ranking
---
- Chicago Cubs
- St Louis Cardinals
```

```
...
```

### Multiple documents 2

```yaml
---
time: 20:03:20
player: Sammy Sosa
action: strike (miss)
---
time: 20:03:47
player: Sammy Sosa
action: grand slam
```

```
...
```
