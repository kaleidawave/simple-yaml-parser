### Keys

```yaml
a: 123
b: 456
c: 789
```

```
[Slice("a")]
 -> Number(123)
[Slice("b")]
 -> Number(456)
[Slice("c")]
 -> Number(789)
```

### Nesting

```yaml
person:
  name: John Doe
  age: 30
  address:
    street: 123 Main St
    city: Example City
```

```
[Slice("person"), Slice("name")]
 -> String("John Doe")
[Slice("person"), Slice("age")]
 -> Number(30)
[Slice("person"), Slice("address"), Slice("street")]
 -> String("123 Main St")
[Slice("person"), Slice("address"), Slice("city")]
 -> String("Example City")
```

### Comments

```yaml
hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In
```

```
[Slice("hr")]
 -> Number(65)
[Slice("avg")]
 -> Number(0.278)
[Slice("rbi")]
 -> Number(147)
```

### List

```yaml
- Mark McGwire
- Sammy Sosa
- Ken Griffey
```

```
[Index { index: 0, bracketed: false }]
 -> String("Mark McGwire")
[Index { index: 1, bracketed: false }]
 -> String("Sammy Sosa")
[Index { index: 2, bracketed: false }]
 -> String("Ken Griffey")
```

### List under key

```yaml
objects:
- a
- b
- c
```

> TODO formatting of index

```
[Slice("objects"), Index { index: 0, bracketed: false }]
 -> String("a")
[Slice("objects"), Index { index: 1, bracketed: false }]
 -> String("b")
[Slice("objects"), Index { index: 2, bracketed: false }]
 -> String("c")
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
[Slice("objects"), Index { index: 0, bracketed: false }, Slice("name")]
 -> String("a")
[Slice("objects"), Index { index: 0, bracketed: false }, Slice("data")]
 -> Number(5)
[Slice("objects"), Index { index: 1, bracketed: false }, Slice("name")]
 -> String("b")
[Slice("objects"), Index { index: 2, bracketed: false }, Slice("name")]
 -> String("c")
[Slice("objects"), Index { index: 2, bracketed: false }, Slice("data")]
 -> Number(7)
```

### Lists of objects (with indent)

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
[Index { index: 0, bracketed: false }, Slice("name")]
 -> String("Mark McGwire")
[Index { index: 0, bracketed: false }, Slice("hr")]
 -> Number(65)
[Index { index: 0, bracketed: false }, Slice("avg")]
 -> Number(0.278)
[Index { index: 1, bracketed: false }, Slice("name")]
 -> String("Sammy Sosa")
[Index { index: 1, bracketed: false }, Slice("hr")]
 -> Number(63)
[Index { index: 1, bracketed: false }, Slice("avg")]
 -> Number(0.288)
```

### Multiline strings

```yaml
objects: |
	this is a paragraph
	that spans multiple lines
```

```
[Slice("objects")]
 -> String("\tthis is a paragraph\n\tthat spans multiple lines")
```

> TODO whitespace here
> `collapse: true, preserve_leading_whitespace: false`

## Expression syntax

### Array literal

```yaml
- [name        , hr, avg  ]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa  , 63, 0.288]
```

```
[Index { index: 0, bracketed: false }, Index { index: 0, bracketed: true }]
 -> String("name")
[Index { index: 0, bracketed: false }, Index { index: 1, bracketed: true }]
 -> String("hr")
[Index { index: 0, bracketed: false }, Index { index: 2, bracketed: true }]
 -> String("avg")
[Index { index: 1, bracketed: false }, Index { index: 0, bracketed: true }]
 -> String("Mark McGwire")
[Index { index: 1, bracketed: false }, Index { index: 1, bracketed: true }]
 -> Number(65)
[Index { index: 1, bracketed: false }, Index { index: 2, bracketed: true }]
 -> Number(0.278)
[Index { index: 2, bracketed: false }, Index { index: 0, bracketed: true }]
 -> String("Sammy Sosa")
[Index { index: 2, bracketed: false }, Index { index: 1, bracketed: true }]
 -> Number(63)
[Index { index: 2, bracketed: false }, Index { index: 2, bracketed: true }]
 -> Number(0.288)
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
[Slice("Mark McGwire"), Slice("hr")]
 -> Number(65)
[Slice("Mark McGwire"), Slice("avg")]
 -> Number(0.278)
[Slice("Sammy Sosa"), Slice("hr")]
 -> Number(63)
[Slice("Sammy Sosa"), Slice("avg")]
 -> Number(0.288)
```

## Multiple documents (skip)

### Multiple documents

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
[Index { index: 0, bracketed: false }]
 -> String("Mark McGwire")
[Index { index: 1, bracketed: false }]
 -> String("Sammy Sosa")
[Index { index: 2, bracketed: false }]
 -> String("Ken Griffey")
[Index { index: 0, bracketed: false }]
 -> String("Chicago Cubs")
[Index { index: 1, bracketed: false }]
 -> String("St Louis Cardinals")
```

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
[Slice("time")]
 -> String("20:03:20")
[Slice("player")]
 -> String("Sammy Sosa")
[Slice("action")]
 -> String("strike (miss)")
[Slice("time")]
 -> String("20:03:47")
[Slice("player")]
 -> String("Sammy Sosa")
[Slice("action")]
 -> String("grand slam")
```
