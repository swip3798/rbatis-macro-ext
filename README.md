# Rbatis Macro Extension

A little procedural macro crate for the ORM rbatis, that combines the simplicity of the `#[sql]` macro with some functionality of the `#[py_sql]` macro.

## Motivation
Disclaimer: I may be wrong on what exactly is possible with the current macros in rbatis. The documentation is not very clear on what is exactly possible and maybe I'm just to dump to figure out, how to do certain things with rbatis itself.

The `#[py_sql]` macro expands to quite a lot of code. Looking at cargo expand, it generates a function with more than 100 lines of code which does a lot more computation that is sometimes necessary (e.g. iterating through every single character of the sql query). On the other side has the `#[py_sql]` macro some great features, like supporting arithmetic operations inside the parameter markers.

The `#[sql]` macro generates way less code, depending on the number of arguments even less than one. This code is very simple: Push all parameters of the function in a `Vec<rbson::Bson>` and use that together with the sql query text to call either fetch or exec on the `Rbatis` object. That's very simple and effective, but lacks functionality. The function you're using the macro on must have exactly the parameters that you want to use in the query, in this exact order. No arithmetic operations nor usage of structs is really possible.

Also both macros don't really allow to create instance functions, that use fields of `self` to query more information (`#[py_sql]` could possibly allow this, but I didn't manage to get it to work). 

## The `ext_sql` macro
This crate offers a new macro that goes a different approach. It is based of the code from the `#[sql]` macro, but offers the `#{var}`-style parameter marks from `#[py_sql]`. The implementation is rather simple, anything you write inside the curly brackets is used as plain rust code that will represent one argument. This can be a parameter of your function, an arithmetic equation, or basically anything that the rust compiler understands. 

Examples:

```
#[ext_sql(DB, "select * from user where name = #{name}")]
pub async fn get_by_name(name: &str) -> User {}

#[ext_sql(DB, "select * from comments where user_id = #{self.id}")]
pub async fn get_by_name(&self) -> Vec<Comment> {}

#[ext_sql(DB, "select * from comments where user_id = #{self.id + 1}")]
pub async fn get_next_user(&self) -> Vec<Comment> {}

#[ext_sql(DB, "select * from updates where due_at = #{rbatis::DateTimeUtc::now()})]
pub async fn get_pending_updates() -> Vec<Update> {}
```
