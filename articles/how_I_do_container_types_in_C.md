---
Title: How I do (type-safe) container types in C
Author: Louis
Date: 2025-08-12
Blurb: type-safe(r) container types
---
# How I do (type-safe) container types in C

Recently, after seeing two articles on how to achieve container-types in C,
I decided I'd also write one.

[Martin Uecker's article](https://uecker.codeberg.page/2025-07-20.html)
| [HN thread](https://uecker.codeberg.page/2025-07-20.html)

[Daniel Hooper's article](https://danielchasehooper.com/posts/typechecked-generic-c-data-structures/)
| [lobste.rs thread](https://lobste.rs/s/s4po4y/how_i_write_type_safe_generic_data)

## Why am I not satisfied with these two articles

The only correct reason is that I suffer from the not-invented-here syndrome,
but I have some complaints that would make me not want to use these implementations.

### Uecker's way

> ```C
> #define vec(T) struct vec_##T { ssize_t N; T data[/* .N */]; }
> ```
>
> -- *Martin Uecker, Generic Containers in C: vec*

This is how I started doing "generics" in C and I quickly ran into issues with
having the macro define the name of the Vec.

For "simple types" this works great, `vec(int)` would expand to:

> ```C
> struct vec_int { ssize_t N, int data[] }
> ```
>
> -- *Martin Uecker, Generic Containers in C: vec*

But for more complex types this would break down pretty quickly.

```C
struct MyValue {int a, int b}

vec(struct MyValue)
```

Would expand to:

```C
struct vec_struct MyValue { ssize_t N, struct MyValue data[] }
```

And it would result in invalid C. This can be worked around by `typedef`-ing the
`struct` instead but it would force me to `typedef` pointers to values and I don't
like how it "pollutes" my namespace. I'm also not that imaginative so if I can
*not* name something I usually go that route.

Overall it's not a bad way of doing things, but my real gripe comes with the
way that the logic is implemented.

> ```C
> #define vec_push(T, v, x)                                  \
>    ({                                                      \
>        vec(T) **_vp = (v);                                 \
>        ssize_t _N = (*_vp)->N + 1;                         \
>        ssize_t _S = _N * (ssize_t)sizeof((*_vp)->data[0])  \
>                + (ssize_t)sizeof(vec(T));                  \
>        if (!(*_vp = realloc(*_vp, _S))) abort();           \
>        (*_vp)->N++;                                        \
>        (*_vp)->data[_N - 1] = (x);                         \
>    })
> ```
>
> -- *Martin Uecker, Generic Containers in C: vec*

Having done that in the past, I now tend to avoid including too much logic
inside my macros because I find that they lead to cryptic error messages and
sometimes variable name clashes. (Again I suck I naming things)
I've spent too much time grep-ing through the output of `cpp` and I've now switched
to doing something else.

### Hooper's way

I find that Hooper does container types in a very similar way to myself.

> ```C
> #define List(type) union { \
>     ListNode *head; \
>     type *payload; \
> }
> ```
>
> -- *Daniel Hooper, Type Safe Generic Data Structures in C*

Defining an unnamed union avoids the complex type problem we ran into with the
other implementation, but as Hooper points out, without doing anything else,
this would result in type errors when expanding the macro more than once.

From Hooper's article:

> ```C
> List(Foo) a;
> List(Foo) b = a; // error
>
> void my_function(List(Foo) list);
> my_function(a); // error: incompatible type
> ```
>
> Even though the variables have identical type definitions, the compiler
> still errors because they are two distinct definitions.
> A `typedef` avoids the issue:
>
> ```C
> typedef List(Foo) ListFoo; // this makes it all work
>
> ListFoo a;
> ListFoo b = a; // ok
>
> void my_function(ListFoo list);
> my_function(a); // ok
>
> List(Foo) local_foo_list; // still works 
> ```
>
> -- *Daniel Hooper, Type Safe Generic Data Structures in C*

I personally don't like how expansions of the same macro won't point back to
the same type. C23 "fixed" this behaviour with it's named record equivalence
rule, but for it to work we would need to make the name of the type part of the
macro and we would run in the same issue with complex types.

## My way

I do it much in the same way as Hooper. I declare a "base implementation"
of my datastructure that every generic version will wrap.

```C
// aligned on eights (or fours on 32 bit machines)
struct Vec {
    size_t len;
    size_t cap;
    void *data;
};
```

And a macro to define a new type of that datastructure.

```C
#define VecDef(_type) \
    typedef struct { \
        struct Vec inner; \
        _type *phantom[0]; \
    }
```

Inserting the typedef directly in the macro allows me to define this type and
to export it rather than re-expanding the same macro every time. The main
drawback of this approach is that typedefs cannot be forward declared but
`structs` can.
Take note that the `phantom` field is a zero sized array of pointers to `_type`,
this way I can forward declare `_type`. Also as a bonus, the zero size array is
a zero-sized type (duh) and, in this case, adds no additional padding.

```C
VecDef(int) IntVec;
VecDef(struct Pos) PosVec;
```

To then get some type safety, I make use of C11's `_Generic` keyword.

```C
#define vecPush(vec, data) _Generic((data), typeof(**((vec)->phantom)): \
    vec_push(&(vec)->inner, sizeof(**((vec)->phantom)), &(data)))
```

By using `_Generic` here I'm able to check that the type passed in matches
exactly the type expected.

```C
VecDef(int) IntVec;
IntVec a = {};

char b = 10;
// Controlling expression type 'char' not compatible
// with any generic association type
vecPush(&a, b);
```

Hooper's way of type checking might be superior since the compiler
will tell you which type it expected instead of just saying your the type is incompatible.

He uses the ternary operator to assert that both the
parameter and the inner type match.

```C
1 ? (param) : *(vec)->type
```

Sadly when it comes to reading a value out, I haven't found a way to have as
much control, I instead cast the pointer, which works great for pointer types,
but I open myself to C type casting rules if I try to dereference that pointer.

```C
#define vecGetPtr(vec, idx) ((typeof(*(vec)->phantom))vec_get_ptr(\
    &(vec)->inner, sizeof(**((vec)->phantom)), idx))

IntVec a = {};
// incompatible pointer types
double *r1 = vecGetPtr(&a, 1);
// C silent type casting, meh
double r2 = *vecGetPtr(&a, 1);
```

This dovetails pretty nicely with C23's `auto` keyword were I basically never
have to worry about type mismatch.

```C
// r3 will always be the correct type
auto r3 = *vecGetPtr(&a, 1);
```

I've found that this technique works pretty well and I've been able to build
all the reusable data-structure I've needed with it:

An Hashmap

```C
#define HMapDef(type) \
    typedef struct { \
        struct HMap inner; \
        type *phantom[0]; \
    }
```

A Queue

```C
#define QueueDef(type) \
    typedef struct { \
        struct Queue inner; \
        type *phantom[0]; \
    }
```

And many more.

The only generic data-structure I use not written in this way is
my implementation of a primary queue, and I'm planning to rewrite it this way in
order to make it type-safe, I just haven't taken the time to do it yet.

[github gist with code examples used in this article](https://gist.github.com/lorlouis/ba227cf544fe917aae0365b41e8c2d04)
