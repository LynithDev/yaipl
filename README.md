<center>

# YAIPL - Yet Another Interpreted Programming Language
Written in Rust so its 🚀 ***blazingly fast*** 🚀

</center>

## Types
YAIPL aims to be a dynamically typed language. Supported types are: `Integer`, `Float`, `Boolean`, `Function`

## Syntax and Keywords
<table>

<tr>
    <th>Keyword</th>
    <th>Description</th>
</tr>

<tr>
    <td>`while`</td>
    <td>Loop through a block of code as long as a specified condition is true</td>
</tr>

<tr>
    <td>`for`</td>
    <td>Loop through a block of code a specified number of times</td>
</tr>

<tr>
    <td>`if`</td>
    <td>Execute a block of code if a specified condition is true</td>
</tr>

</table>

### Assignment
Creating and reassigning variables is done using the `=` operator.
```py
# Assignining variables
my_variable = 5

# Reassigning variables
my_variable = 10
```

You can also create and REASSIGN functions using the `=` operator.
```py
# Defining a function
my_function = (param) {
    param + 5 # Implicit return
}

# Reassigning a function
my_function = (param) {
    param * 5
}

# Calling the function
print(my_function(5)) # returns 25
```

### Standard Library
The standard library contains a collection of functions accessible globally. Functions such as `print` are accessible through this library.
