//! isu is a Rust implementation of Information State Update theory.
//! The library can be use for Issue-Based Dialogue Management and 
//! Conversational Agent Architecture.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::io::{self, Write};
use std::hash::Hash;
use std::any::Any;

// Input handling traits and implementations

/// Trait for input handling abstraction
pub trait InputHandler {
    /// Attempts to read a line of input
    /// Returns None if no input is available or on EOF
    fn read_line(&mut self) -> Option<String>;
    
    /// Returns true if input is available
    fn has_input(&self) -> bool;
}

/// Standard input handler that blocks for user input
pub struct StandardInputHandler;

impl InputHandler for StandardInputHandler {
    fn read_line(&mut self) -> Option<String> {
        print!("U> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => Some(input.trim().to_string()),
            Err(_) => {
                println!("EOF");
                None
            }
        }
    }
    
    fn has_input(&self) -> bool {
        true // Always assume input is available for blocking input
    }
}

/// Demo input handler with predefined inputs
pub struct DemoInputHandler {
    inputs: VecDeque<String>,
    current_index: usize,
}

impl DemoInputHandler {
    pub fn new(inputs: Vec<String>) -> Self {
        Self {
            inputs: inputs.into(),
            current_index: 0,
        }
    }
}

impl InputHandler for DemoInputHandler {
    fn read_line(&mut self) -> Option<String> {
        if let Some(input) = self.inputs.pop_front() {
            println!("U> {}", input); // Show simulated user input
            Some(input)
        } else {
            println!("Demo completed - no more inputs");
            None
        }
    }
    
    fn has_input(&self) -> bool {
        !self.inputs.is_empty()
    }
}

// Helper functions

/// Checks if a given type can be treated as a sequence.
/// Note: Simplified to always return true due to Rust's type system constraints.
/// Modify based on specific type requirements.
fn is_sequence<T>(seq: &T) -> bool {
    // This is a simplified version assuming we work with specific types.
    true // Modify based on specific needs
}

// Value struct

/// A generic container for values with constraints on allowed values or type checks.
/// Represents a single value with optional validation rules.
struct Value<T: Clone + PartialEq + Eq + Hash> {
    value: Option<T>, // The stored value, if any
    allowed_values: HashSet<T>, // Set of permitted values
    type_constraint: Option<Box<dyn Fn(&T) -> bool>>, // Optional type checking function
}

impl<T: Clone + PartialEq + Eq + Hash> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            value: self.value.clone(),
            allowed_values: self.allowed_values.clone(),
            type_constraint: None, // Cannot clone function pointers
        }
    }
}

/// Implementation of methods for the Value struct.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> Value<T> {
    /// Creates a new Value with a set of allowed values.
    /// # Arguments
    /// * `allowed` - A HashSet of permitted values.
    fn new_allowed(allowed: HashSet<T>) -> Self {
        Value {
            value: None,
            allowed_values: allowed,
            type_constraint: None,
        }
    }

    /// Creates a new Value with a type constraint function.
    /// # Arguments
    /// * `type_check` - A function to validate the type of values.
    fn new_type<F>(type_check: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Value {
            value: None,
            allowed_values: HashSet::new(),
            type_constraint: Some(Box::new(type_check)),
        }
    }

    /// Sets the value after validating against constraints.
    /// Returns an error if the value is not allowed or fails the type check.
    /// # Arguments
    /// * `value` - The value to set.
    fn set(&mut self, value: T) -> Result<(), String> {
        if !self.allowed_values.is_empty() && !self.allowed_values.contains(&value) {
            return Err(format!("{} is not among allowed values", value));
        }
        if let Some(check) = &self.type_constraint {
            if !check(&value) {
                return Err(format!("{} does not match type constraint", value));
            }
        }
        self.value = Some(value);
        Ok(())
    }

    /// Retrieves the stored value, if any.
    fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Clears the stored value.
    fn clear(&mut self) {
        self.value = None;
    }
}

/// Formats the Value for display, showing the stored value or an empty marker.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> fmt::Display for Value<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.value {
            Some(v) => write!(f, "<{}>", v),
            None => write!(f, "<>"),
        }
    }
}

// Record struct

/// A key-value store with type checking for fields.
struct Record {
    typedict: HashMap<String, Box<dyn Fn(&dyn Any) -> bool>>, // Type checking functions for fields
    fields: HashMap<String, Box<dyn Any>>, // Stored field values
}

impl Clone for Record {
    fn clone(&self) -> Self {
        Record {
            typedict: HashMap::new(), // Cannot clone function pointers
            fields: HashMap::new(), // Cannot clone Any trait objects safely
        }
    }
}

/// Implementation of methods for the Record struct.
impl Record {
    /// Creates a new Record with initial fields and inferred type checks.
    /// # Arguments
    /// * `fields` - Initial key-value pairs.
    fn new(fields: HashMap<String, Box<dyn Any>>) -> Self {
        let mut typedict: HashMap<String, Box<dyn Fn(&dyn Any) -> bool>> = HashMap::new();
        for (key, value) in &fields {
            let type_id = value.type_id();
            typedict.insert(key.clone(), Box::new(move |v: &dyn Any| v.type_id() == type_id) as Box<dyn Fn(&dyn Any) -> bool>);
        }
        Record { typedict, fields }
    }

    /// Returns a HashMap of field keys to their values.
    fn as_dict(&self) -> HashMap<String, &dyn Any> {
        self.fields.iter().map(|(k, v)| (k.clone(), v.as_ref())).collect()
    }

    /// Checks if a value matches the expected type for a given key.
    /// # Arguments
    /// * `key` - The field key to check.
    /// * `value` - Optional value to type check.
    fn typecheck(&self, key: &str, value: Option<&dyn Any>) -> Result<(), String> {
        if let Some(type_fn) = self.typedict.get(key) {
            if let Some(val) = value {
                if !type_fn(val) {
                    return Err(format!("{} is not of expected type", key));
                }
            }
            Ok(())
        } else {
            Err(format!("{} is not a valid key", key))
        }
    }

    /// Retrieves a field value by key after type checking.
    /// # Arguments
    /// * `key` - The field key.
    fn get(&self, key: &str) -> Option<&dyn Any> {
        self.typecheck(key, None).ok()?;
        self.fields.get(key).map(|v| v.as_ref())
    }

    /// Sets a field value after type checking.
    /// # Arguments
    /// * `key` - The field key.
    /// * `value` - The value to set.
    fn set(&mut self, key: &str, value: Box<dyn Any>) -> Result<(), String> {
        self.typecheck(key, Some(value.as_ref()))?;
        self.fields.insert(key.to_string(), value);
        Ok(())
    }

    /// Removes a field by key after type checking.
    /// # Arguments
    /// * `key` - The field key to remove.
    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.typecheck(key, None)?;
        self.fields.remove(key);
        Ok(())
    }

    /// Formats the Record as a string with a given prefix and indent.
    /// # Arguments
    /// * `prefix` - Prefix for each line.
    /// * `indent` - Indentation string (unused in this implementation).
    fn pformat(&self, prefix: &str, indent: &str) -> String {
        let mut result = String::new();
        for (key, value) in self.as_dict() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(prefix);
            result.push_str(&key);
            result.push_str(": ");
            // Simplified: assumes value can be formatted as string
            result.push_str(&format!("{:?}", value));
        }
        result
    }
}

/// Formats the Record for display as a semicolon-separated list of key-value pairs.
impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let items: Vec<String> = self.as_dict().iter().map(|(k, v)| format!("{} = {:?}", k, v)).collect();
        write!(f, "{{{}}}", items.join("; "))
    }
}

// Stack struct

/// A generic stack with optional type constraints.
struct Stack<T: Clone> {
    elements: Vec<T>, // The stack's elements
    type_constraint: Option<Box<dyn Fn(&T) -> bool>>, // Optional type checking function
}

impl<T: Clone> Clone for Stack<T> {
    fn clone(&self) -> Self {
        Stack {
            elements: self.elements.clone(),
            type_constraint: None, // Cannot clone function pointers
        }
    }
}

/// Implementation of methods for the Stack struct.
impl<T: Clone + PartialEq + fmt::Display> Stack<T> {
    /// Creates a new empty Stack.
    fn new() -> Self {
        Stack {
            elements: Vec::new(),
            type_constraint: None,
        }
    }

    /// Creates a new Stack with a type constraint.
    /// # Arguments
    /// * `type_check` - A function to validate pushed values.
    fn with_type<F>(type_check: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        Stack {
            elements: Vec::new(),
            type_constraint: Some(Box::new(type_check)),
        }
    }

    /// Returns a reference to the top element of the stack.
    fn top(&self) -> Result<&T, String> {
        self.elements.last().ok_or("Stack is empty".to_string())
    }

    /// Removes and returns the top element of the stack.
    fn pop(&mut self) -> Result<T, String> {
        self.elements.pop().ok_or("Stack is empty".to_string())
    }

    /// Pushes a value onto the stack after type checking.
    /// # Arguments
    /// * `value` - The value to push.
    fn push(&mut self, value: T) -> Result<(), String> {
        if let Some(check) = &self.type_constraint {
            if !check(&value) {
                return Err(format!("{} does not match type constraint", value));
            }
        }
        self.elements.push(value);
        Ok(())
    }

    /// Clears all elements from the stack.
    fn clear(&mut self) {
        self.elements.clear();
    }

    /// Returns the number of elements in the stack.
    fn len(&self) -> usize {
        self.elements.len()
    }
}

/// Formats the Stack for display, showing elements in reverse order.
impl<T: Clone + fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reversed: Vec<String> = self.elements.iter().rev().map(|e| e.to_string()).collect();
        write!(f, "<[ {} <]", reversed.join(", "))
    }
}

// StackSet struct

/// A stack-based set ensuring unique elements with LIFO order.
#[derive(Clone)]
struct StackSet<T: Clone + PartialEq + Eq + Hash> {
    stack: Stack<T>, // Underlying stack for storage
}

/// Implementation of methods for the StackSet struct.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> StackSet<T> {
    /// Creates a new empty StackSet.
    fn new() -> Self {
        StackSet { stack: Stack::new() }
    }

    /// Creates a new StackSet with a type constraint.
    /// # Arguments
    /// * `type_check` - A function to validate elements.
    fn with_type<F>(type_check: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        StackSet {
            stack: Stack::with_type(type_check),
        }
    }

    /// Checks if a value is present in the StackSet.
    /// # Arguments
    /// * `value` - The value to check.
    fn contains(&self, value: &T) -> bool {
        self.stack.elements.contains(value)
    }

    /// Pushes a value, removing any existing instance to maintain uniqueness.
    /// # Arguments
    /// * `value` - The value to push.
    fn push(&mut self, value: T) -> Result<(), String> {
        if self.contains(&value) {
            self.stack.elements.retain(|x| x != &value);
        }
        self.stack.push(value)
    }
}

/// Formats the StackSet for display, showing the underlying stack.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> fmt::Display for StackSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{{ {} <}}", self.stack.to_string())
    }
}

// TSet struct

/// A typed set with optional type constraints for elements.
struct TSet<T: Clone + PartialEq + Eq + Hash> {
    elements: HashSet<T>, // The set of elements
    type_constraint: Option<Box<dyn Fn(&T) -> bool>>, // Optional type checking function
}

impl<T: Clone + PartialEq + Eq + Hash> Clone for TSet<T> {
    fn clone(&self) -> Self {
        TSet {
            elements: self.elements.clone(),
            type_constraint: None, // Cannot clone function pointers
        }
    }
}

/// Implementation of methods for the TSet struct.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> TSet<T> {
    /// Creates a new empty TSet.
    fn new() -> Self {
        TSet {
            elements: HashSet::new(),
            type_constraint: None,
        }
    }

    /// Creates a new TSet with a type constraint.
    /// # Arguments
    /// * `type_check` - A function to validate elements.
    fn with_type<F>(type_check: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        TSet {
            elements: HashSet::new(),
            type_constraint: Some(Box::new(type_check)),
        }
    }

    /// Adds an element to the TSet after type checking.
    /// # Arguments
    /// * `value` - The value to add.
    fn add(&mut self, value: T) -> Result<(), String> {
        if let Some(check) = &self.type_constraint {
            if !check(&value) {
                return Err(format!("{} does not match type constraint", value));
            }
        }
        self.elements.insert(value);
        Ok(())
    }

    /// Clears all elements from the TSet.
    fn clear(&mut self) {
        self.elements.clear();
    }

    /// Returns the number of elements in the TSet.
    fn len(&self) -> usize {
        self.elements.len()
    }

    /// Checks if a value is present in the TSet.
    /// # Arguments
    /// * `value` - The value to check.
    fn contains(&self, value: &T) -> bool {
        self.elements.contains(value)
    }
}

/// Formats the TSet for display as a comma-separated list of elements.
impl<T: Clone + PartialEq + Eq + Hash + fmt::Display> fmt::Display for TSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let elements: Vec<String> = self.elements.iter().map(|e| e.to_string()).collect();
        write!(f, "{{{}}}", elements.join(", "))
    }
}

// Enum creation macro

/// Macro to create an enum with string parsing and display capabilities.
/// # Arguments
/// * `$name` - The name of the enum.
/// * `$($variant),+` - The variants of the enum.
macro_rules! create_enum {
    ($name:ident, $($variant:ident),+) => {
        /// An enumeration with named variants.
        #[derive(Clone, PartialEq, Eq, Debug, Hash)]
        enum $name {
            $($variant),+
        }

        impl $name {
            /// Creates an enum variant from a string name.
            /// # Arguments
            /// * `name` - The string name of the variant.
            fn new(name: &str) -> Option<Self> {
                match name {
                    $(stringify!($variant) => Some($name::$variant),)+
                    _ => None,
                }
            }
        }

        /// Formats the enum for display.
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $($name::$variant => write!(f, "{}", stringify!($variant)),)+
                }
            }
        }
    };
}

// Define Speaker and ProgramState enums
create_enum!(Speaker, USR, SYS);
create_enum!(ProgramState, RUN, QUIT);

// Semantic types

/// Trait for types that can be type-checked within a Domain.
trait Type: fmt::Display {
    /// Checks if the type is valid within the given Domain context.
    /// # Arguments
    /// * `context` - The Domain to check against.
    fn typecheck(&self, context: &Domain) -> Result<(), String>;
}

/// Represents an atomic string with validation rules.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Atomic {
    content: String, // The atomic string value
}

/// Implementation of methods for the Atomic struct.
impl Atomic {
    /// Creates a new Atomic value with validation.
    /// # Arguments
    /// * `atom` - The string to validate and store.
    fn new(atom: &str) -> Result<Self, String> {
        if atom.is_empty() || atom == "yes" || atom == "no" {
            return Err("Invalid atom".to_string());
        }
        if !atom.chars().next().unwrap_or(' ').is_alphabetic() {
            return Err("Atom must start with a letter".to_string());
        }
        if !atom.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '+' || c == ':') {
            return Err("Invalid characters in atom".to_string());
        }
        Ok(Atomic { content: atom.to_string() })
    }
}

/// Formats the Atomic value for display.
impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Represents an individual in the domain, wrapping an Atomic value.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Ind(Atomic);

/// Implementation of methods for the Ind struct.
impl Ind {
    /// Creates a new Ind from a string.
    /// # Arguments
    /// * `atom` - The string to create an Atomic value from.
    fn new(atom: &str) -> Result<Self, String> {
        Ok(Ind(Atomic::new(atom)?))
    }
}

/// Implements type checking for Ind against a Domain.
impl Type for Ind {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        if context.inds.contains_key(&self.0.content) {
            Ok(())
        } else {
            Err(format!("{} not in context individuals", self.0.content))
        }
    }
}

/// Formats the Ind for display.
impl fmt::Display for Ind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a zero-place predicate.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Pred0(Atomic);

/// Implementation of methods for the Pred0 struct.
impl Pred0 {
    /// Creates a new Pred0 from a string.
    /// # Arguments
    /// * `atom` - The string to create an Atomic value from.
    fn new(atom: &str) -> Result<Self, String> {
        Ok(Pred0(Atomic::new(atom)?))
    }
}

/// Implements type checking for Pred0 against a Domain.
impl Type for Pred0 {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        if context.preds0.contains(&self.0.content) {
            Ok(())
        } else {
            Err(format!("{} not in context 0-place predicates", self.0.content))
        }
    }
}

/// Formats the Pred0 for display.
impl fmt::Display for Pred0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a one-place predicate.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Pred1(Atomic);

/// Implementation of methods for the Pred1 struct.
impl Pred1 {
    /// Creates a new Pred1 from a string.
    /// # Arguments
    /// * `atom` - The string to create an Atomic value from.
    fn new(atom: &str) -> Result<Self, String> {
        Ok(Pred1(Atomic::new(atom)?))
    }

    /// Applies the predicate to an individual to create a proposition.
    /// # Arguments
    /// * `ind` - The individual to apply the predicate to.
    fn apply(&self, ind: &Ind) -> Result<Prop, Box<dyn std::error::Error>> {
        Ok(Prop {
            pred: Pred0::new(&self.0.content)?,
            ind: Some(ind.clone()),
            yes: true,
        })
    }
}

/// Implements type checking for Pred1 against a Domain.
impl Type for Pred1 {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        if context.preds1.contains_key(&self.0.content) {
            Ok(())
        } else {
            Err(format!("{} not in context 1-place predicates", self.0.content))
        }
    }
}

/// Formats the Pred1 for display.
impl fmt::Display for Pred1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a sort (category) for individuals, wrapping a Pred1.
#[derive(Clone)]
struct Sort(Pred1);

/// Implementation of methods for the Sort struct.
impl Sort {
    /// Creates a new Sort from a string.
    /// # Arguments
    /// * `atom` - The string to create a Pred1 from.
    fn new(atom: &str) -> Result<Self, String> {
        Ok(Sort(Pred1::new(atom)?))
    }
}

/// Implements type checking for Sort against a Domain.
impl Type for Sort {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        if context.sorts.contains_key(&self.0 .0.content) {
            Ok(())
        } else {
            Err(format!("{} not in context sorts", self.0 .0.content))
        }
    }
}

/// Formats the Sort for display.
impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a proposition, combining a predicate with an optional individual and polarity.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Prop {
    pred: Pred0, // The predicate
    ind: Option<Ind>, // Optional individual
    yes: bool, // Polarity (true for positive, false for negative)
}

/// Implementation of methods for the Prop struct.
impl Prop {
    /// Creates a new Prop from a string, parsing polarity and arguments.
    /// # Arguments
    /// * `s` - The string to parse (e.g., "pred(ind)" or "-pred").
    fn new(s: &str) -> Result<Self, String> {
        let (yes, pred_str, ind_str) = if s.starts_with('-') {
            (false, &s[1..], None::<&str>)
        } else {
            (true, s, None)
        };
        let (pred_str, ind_str) = if pred_str.ends_with(')') {
            let parts: Vec<&str> = pred_str[..pred_str.len() - 1].split('(').collect();
            if parts.len() == 2 {
                (parts[0], Some(parts[1]))
            } else {
                (pred_str, None)
            }
        } else {
            (pred_str, None)
        };
        let pred = if ind_str.is_some() {
            Pred0::new(pred_str)? // Simplified: assuming Pred0 for now
        } else {
            Pred0::new(pred_str)?
        };
        let ind = ind_str.map(|s| Ind::new(s).unwrap());
        Ok(Prop { pred, ind, yes })
    }
}

/// Implements type checking for Prop against a Domain.
impl Type for Prop {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.pred.typecheck(context)?;
        if let Some(ind) = &self.ind {
            ind.typecheck(context)?;
            if let Some(sort) = context.preds1.get(&self.pred.0.content) {
                if context.inds.get(&ind.0.content) != Some(sort) {
                    return Err("Sort mismatch".to_string());
                }
            }
        }
        Ok(())
    }
}

/// Formats the Prop for display, including polarity and arguments.
impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = if self.yes { "" } else { "-" };
        let ind_str = self.ind.as_ref().map_or("", |ind| &ind.0.content);
        write!(f, "{}{}({})", prefix, self.pred, ind_str)
    }
}

/// Represents a short answer (e.g., "paris" or "-paris").
#[derive(Clone)]
struct ShortAns {
    ind: Ind, // The individual
    yes: bool, // Polarity
}

/// Implementation of methods for the ShortAns struct.
impl ShortAns {
    /// Creates a new ShortAns from a string, parsing polarity.
    /// # Arguments
    /// * `s` - The string to parse.
    fn new(s: &str) -> Result<Self, String> {
        let (yes, ind_str) = if s.starts_with('-') {
            (false, &s[1..])
        } else {
            (true, s)
        };
        Ok(ShortAns {
            ind: Ind::new(ind_str)?,
            yes,
        })
    }
}

/// Implements type checking for ShortAns against a Domain.
impl Type for ShortAns {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.ind.typecheck(context)
    }
}

/// Formats the ShortAns for display.
impl fmt::Display for ShortAns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = if self.yes { "" } else { "-" };
        write!(f, "{}{}", prefix, self.ind)
    }
}

/// Represents a yes/no answer.
#[derive(Clone)]
struct YesNo {
    yes: bool, // True for "yes", false for "no"
}

/// Implementation of methods for the YesNo struct.
impl YesNo {
    /// Creates a new YesNo from a string.
    /// # Arguments
    /// * `s` - The string ("yes" or "no").
    fn new(s: &str) -> Result<Self, String> {
        match s {
            "yes" => Ok(YesNo { yes: true }),
            "no" => Ok(YesNo { yes: false }),
            _ => Err(format!("Invalid YesNo: {}", s)),
        }
    }
}

/// Implements type checking for YesNo (always valid).
impl Type for YesNo {
    fn typecheck(&self, _context: &Domain) -> Result<(), String> {
        Ok(())
    }
}

/// Formats the YesNo for display.
impl fmt::Display for YesNo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if self.yes { "yes" } else { "no" })
    }
}

/// Enum representing different types of answers.
#[derive(Clone)]
enum Ans {
    Prop(Prop), // A proposition
    ShortAns(ShortAns), // A short answer
    YesNo(YesNo), // A yes/no answer
}

/// Implementation of methods for the Ans enum.
impl Ans {
    /// Creates a new Ans from a string, parsing the appropriate type.
    /// # Arguments
    /// * `s` - The string to parse.
    fn new(s: &str) -> Result<Self, String> {
        if s == "yes" || s == "no" {
            Ok(Ans::YesNo(YesNo::new(s)?))
        } else if !s.contains('(') && !s.contains(')') {
            Ok(Ans::ShortAns(ShortAns::new(s)?))
        } else if s.contains('(') && s.ends_with(')') {
            Ok(Ans::Prop(Prop::new(s)?))
        } else {
            Err(format!("Could not parse answer: {}", s))
        }
    }
}

/// Implements type checking for Ans against a Domain.
impl Type for Ans {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        match self {
            Ans::Prop(p) => p.typecheck(context),
            Ans::ShortAns(s) => s.typecheck(context),
            Ans::YesNo(y) => y.typecheck(context),
        }
    }
}

/// Formats the Ans for display.
impl fmt::Display for Ans {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ans::Prop(p) => write!(f, "{}", p),
            Ans::ShortAns(s) => write!(f, "{}", s),
            Ans::YesNo(y) => write!(f, "{}", y),
        }
    }
}

/// Represents a "wh" question (e.g., "?x.pred(x)").
#[derive(Clone)]
struct WhQ {
    pred: Pred1, // The predicate
}

/// Implementation of methods for the WhQ struct.
impl WhQ {
    /// Creates a new WhQ from a string, parsing the predicate.
    /// # Arguments
    /// * `pred` - The predicate string (e.g., "?x.pred(x)" or "pred").
    fn new(pred: &str) -> Result<Self, String> {
        let pred = if pred.starts_with("?x.") && pred.ends_with("(x)") {
            &pred[3..pred.len() - 3]
        } else {
            pred
        };
        Ok(WhQ {
            pred: Pred1::new(pred)?,
        })
    }
}

/// Implements type checking for WhQ against a Domain.
impl Type for WhQ {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.pred.typecheck(context)
    }
}

/// Formats the WhQ for display.
impl fmt::Display for WhQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "?x.{} (x)", self.pred)
    }
}

/// Represents a yes/no question.
#[derive(Clone)]
struct YNQ {
    prop: Prop, // The proposition
}

/// Implementation of methods for the YNQ struct.
impl YNQ {
    /// Creates a new YNQ from a string.
    /// # Arguments
    /// * `prop` - The proposition string (e.g., "?pred(ind)").
    fn new(prop: &str) -> Result<Self, String> {
        let prop = if prop.starts_with('?') { &prop[1..] } else { prop };
        Ok(YNQ {
            prop: Prop::new(prop)?,
        })
    }
}

/// Implements type checking for YNQ against a Domain.
impl Type for YNQ {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.prop.typecheck(context)
    }
}

/// Formats the YNQ for display.
impl fmt::Display for YNQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "?{}", self.prop)
    }
}

/// Represents an alternative question (multiple yes/no questions).
#[derive(Clone)]
struct AltQ {
    ynqs: Vec<YNQ>, // List of yes/no questions
}

/// Implementation of methods for the AltQ struct.
impl AltQ {
    /// Creates a new AltQ from a list of YNQs.
    /// # Arguments
    /// * `ynqs` - The list of yes/no questions.
    fn new(ynqs: Vec<YNQ>) -> Self {
        AltQ { ynqs }
    }
}

/// Implements type checking for AltQ against a Domain.
impl Type for AltQ {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        for ynq in &self.ynqs {
            ynq.typecheck(context)?;
        }
        Ok(())
    }
}

/// Formats the AltQ for display.
impl fmt::Display for AltQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ynq_strs: Vec<String> = self.ynqs.iter().map(|y| y.to_string()).collect();
        write!(f, "{{ {} }}", ynq_strs.join(" | "))
    }
}

/// Enum representing different types of questions.
#[derive(Clone)]
pub enum Question {
    WhQ(WhQ), // Wh-question
    YNQ(YNQ), // Yes/no question
    AltQ(AltQ), // Alternative question
}

/// Implementation of methods for the Question enum.
impl Question {
    /// Creates a new Question from a string.
    /// # Arguments
    /// * `s` - The string to parse.
    pub fn new(s: &str) -> Result<Self, String> {
        if s.starts_with("?x.") && s.ends_with("(x)") {
            Ok(Question::WhQ(WhQ::new(&s[3..s.len() - 3])?))
        } else if s.starts_with('?') {
            Ok(Question::YNQ(YNQ::new(&s[1..])?))
        } else {
            Err(format!("Could not parse question: {}", s))
        }
    }
}

/// Implements type checking for Question against a Domain.
impl Type for Question {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        match self {
            Question::WhQ(w) => w.typecheck(context),
            Question::YNQ(y) => y.typecheck(context),
            Question::AltQ(a) => a.typecheck(context),
        }
    }
}

/// Formats the Question for display.
impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Question::WhQ(w) => write!(f, "{}", w),
            Question::YNQ(y) => write!(f, "{}", y),
            Question::AltQ(a) => write!(f, "{}", a),
        }
    }
}

// Dialogue moves

/// Represents a greeting dialogue move.
#[derive(Clone)]
struct Greet;

/// Implements type checking for Greet (always valid).
impl Type for Greet {
    fn typecheck(&self, _context: &Domain) -> Result<(), String> {
        Ok(())
    }
}

/// Formats the Greet for display.
impl fmt::Display for Greet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Greet()")
    }
}

/// Represents a quit dialogue move.
#[derive(Clone)]
struct Quit;

/// Implements type checking for Quit (always valid).
impl Type for Quit {
    fn typecheck(&self, _context: &Domain) -> Result<(), String> {
        Ok(())
    }
}

/// Formats the Quit for display.
impl fmt::Display for Quit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Quit()")
    }
}

/// Represents an ask dialogue move.
#[derive(Clone)]
struct Ask {
    content: Question, // The question being asked
}

/// Implementation of methods for the Ask struct.
impl Ask {
    /// Creates a new Ask move.
    /// # Arguments
    /// * `content` - The question to ask.
    fn new(content: Question) -> Self {
        Ask { content }
    }
}

/// Implements type checking for Ask against a Domain.
impl Type for Ask {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the Ask for display.
impl fmt::Display for Ask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ask('{}')", self.content)
    }
}

/// Represents an answer dialogue move.
#[derive(Clone)]
struct Answer {
    content: Ans, // The answer content
}

/// Implementation of methods for the Answer struct.
impl Answer {
    /// Creates a new Answer move.
    /// # Arguments
    /// * `content` - The answer content.
    fn new(content: Ans) -> Self {
        Answer { content }
    }
}

/// Implements type checking for Answer against a Domain.
impl Type for Answer {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the Answer for display.
impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Answer({})", self.content)
    }
}

/// Represents an Information State Update Control Mechanism (ICM).
#[derive(Clone)]
struct ICM {
    level: String, // The ICM level (e.g., "per" for perception)
    polarity: String, // The polarity (e.g., "pos" or "neg")
    icm_content: Option<String>, // Optional content for the ICM
}

/// Implementation of methods for the ICM struct.
impl ICM {
    /// Creates a new ICM.
    /// # Arguments
    /// * `level` - The ICM level.
    /// * `polarity` - The polarity.
    /// * `icm_content` - Optional content.
    fn new(level: &str, polarity: &str, icm_content: Option<String>) -> Self {
        ICM {
            level: level.to_string(),
            polarity: polarity.to_string(),
            icm_content,
        }
    }
}

/// Implements type checking for ICM (always valid).
impl Type for ICM {
    fn typecheck(&self, _context: &Domain) -> Result<(), String> {
        Ok(())
    }
}

/// Formats the ICM for display.
impl fmt::Display for ICM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = format!("icm:{}*{}", self.level, self.polarity);
        if let Some(content) = &self.icm_content {
            s.push_str(&format!(":'{}'", content));
        }
        write!(f, "{}", s)
    }
}

// Plan constructors

/// Represents a respond plan constructor.
#[derive(Clone)]
struct Respond {
    content: Question, // The question to respond to
}

/// Implementation of methods for the Respond struct.
impl Respond {
    /// Creates a new Respond plan.
    /// # Arguments
    /// * `content` - The question to respond to.
    fn new(content: Question) -> Self {
        Respond { content }
    }
}

/// Implements type checking for Respond against a Domain.
impl Type for Respond {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the Respond for display.
impl fmt::Display for Respond {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Respond('{}')", self.content)
    }
}

/// Represents a consult database plan constructor.
#[derive(Clone)]
pub struct ConsultDB {
    content: Question, // The question to consult
}

/// Implementation of methods for the ConsultDB struct.
impl ConsultDB {
    /// Creates a new ConsultDB plan.
    /// # Arguments
    /// * `content` - The question to consult.
    pub fn new(content: Question) -> Self {
        ConsultDB { content }
    }
}

/// Implements type checking for ConsultDB against a Domain.
impl Type for ConsultDB {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the ConsultDB for display.
impl fmt::Display for ConsultDB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConsultDB('{}')", self.content)
    }
}

/// Represents a findout plan constructor.
#[derive(Clone)]
pub struct Findout {
    content: Question, // The question to find out
}

/// Implementation of methods for the Findout struct.
impl Findout {
    /// Creates a new Findout plan.
    /// # Arguments
    /// * `content` - The question to find out.
    pub fn new(content: Question) -> Self {
        Findout { content }
    }
}

/// Implements type checking for Findout against a Domain.
impl Type for Findout {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the Findout for display.
impl fmt::Display for Findout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Findout('{}')", self.content)
    }
}

/// Represents a raise plan constructor.
#[derive(Clone)]
struct Raise {
    content: Question, // The question to raise
}

/// Implementation of methods for the Raise struct.
impl Raise {
    /// Creates a new Raise plan.
    /// # Arguments
    /// * `content` - The question to raise.
    fn new(content: Question) -> Self {
        Raise { content }
    }
}

/// Implements type checking for Raise against a Domain.
impl Type for Raise {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.content.typecheck(context)
    }
}

/// Formats the Raise for display.
impl fmt::Display for Raise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Raise('{}')", self.content)
    }
}

/// Represents a conditional plan constructor.
#[derive(Clone)]
pub struct If {
    cond: Question, // The condition question
    iftrue: Vec<String>, // Plans if condition is true
    iffalse: Vec<String>, // Plans if condition is false
}

/// Implementation of methods for the If struct.
impl If {
    /// Creates a new If plan.
    /// # Arguments
    /// * `cond` - The condition question.
    /// * `iftrue` - Plans to execute if true.
    /// * `iffalse` - Plans to execute if false.
    pub fn new(cond: Question, iftrue: Vec<String>, iffalse: Vec<String>) -> Self {
        If { cond, iftrue, iffalse }
    }
}

/// Implements type checking for If against a Domain.
impl Type for If {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.cond.typecheck(context)?;
        Ok(())
    }
}

/// Formats the If for display.
impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iftrue_str: Vec<String> = self.iftrue.clone();
        let iffalse_str: Vec<String> = self.iffalse.clone();
        write!(f, "If('{}', {}, {})", self.cond, iftrue_str.join(", "), iffalse_str.join(", "))
    }
}

/// Trait for plan constructors.
pub trait PlanConstructor: Type + fmt::Display + Clone {}

impl PlanConstructor for Respond {}
impl PlanConstructor for ConsultDB {}
impl PlanConstructor for Findout {}
impl PlanConstructor for Raise {}
impl PlanConstructor for If {}

// Dialogue Manager

/// Trait for managing dialogue flow and state.
trait DialogueManager {
    /// Logs a trace message.
    /// # Arguments
    /// * `message` - The message to log.
    fn trace(&self, message: &str) {
        println!("{{{}}}", message);
    }

    /// Runs the dialogue manager.
    fn run(&mut self) {
        self.reset();
        self.control();
    }

    /// Resets the dialogue state.
    fn reset(&mut self);

    /// Controls the dialogue flow.
    fn control(&mut self);

    /// Prints the current dialogue state.
    fn print_state(&self);
}

/// Standard MIVS (Minimal Information State) for dialogue management.
struct StandardMIVS {
    input: Value<String>, // User input
    latest_speaker: Value<Speaker>, // Latest speaker (USR or SYS)
    latest_moves: TSet<String>, // Latest dialogue moves
    next_moves: Stack<String>, // Next moves to perform
    output: Value<String>, // System output
    program_state: Value<ProgramState>, // Program state (RUN or QUIT)
}

/// Implementation of methods for the StandardMIVS struct.
impl StandardMIVS {
    /// Initializes the MIVS state.
    fn init_mivs(&mut self) {
        self.input = Value::new_type(|_: &String| true);
        self.latest_speaker = Value::new_allowed(HashSet::from([Speaker::USR, Speaker::SYS]));
        self.latest_moves = TSet::new();
        self.next_moves = Stack::new();
        self.output = Value::new_type(|_: &String| true);
        self.program_state = Value::new_allowed(HashSet::from([ProgramState::RUN, ProgramState::QUIT]));
        self.program_state.set(ProgramState::RUN).unwrap();
    }

    /// Prints the MIVS state with a prefix.
    /// # Arguments
    /// * `prefix` - The prefix for each line.
    fn print_mivs(&self, prefix: &str) {
        println!("{}INPUT:          {}", prefix, self.input);
        println!("{}LATEST_SPEAKER: {}", prefix, self.latest_speaker);
        println!("{}LATEST_MOVES:   {}", prefix, self.latest_moves);
        println!("{}NEXT_MOVES:     {}", prefix, self.next_moves);
        println!("{}OUTPUT:         {}", prefix, self.output);
        println!("{}PROGRAM_STATE:  {}", prefix, self.program_state);
    }
}

// Grammar

/// Trait for generating and interpreting dialogue moves.
trait Grammar {
    /// Generates a string from a set of moves.
    /// # Arguments
    /// * `moves` - The set of moves to generate.
    fn generate(&self, moves: &TSet<String>) -> String;

    /// Interprets an input string into a set of moves.
    /// # Arguments
    /// * `input` - The input string to interpret.
    fn interpret(&self, input: &str) -> Option<TSet<String>>;
}

/// A simple grammar for generating and interpreting dialogue moves.
pub struct SimpleGenGrammar {
    forms: HashMap<String, String>, // Mapping of move strings to output strings
}

/// Implementation of methods for the SimpleGenGrammar struct.
impl SimpleGenGrammar {
    /// Creates a new SimpleGenGrammar with default forms.
    pub fn new() -> Self {
        let mut grammar = SimpleGenGrammar {
            forms: HashMap::new(),
        };
        grammar.add_form("Greet()", "Hello");
        grammar.add_form("icm:neg*sem", "I don't understand");
        grammar
    }

    /// Adds a form to the grammar.
    /// # Arguments
    /// * `move_str` - The move string.
    /// * `output` - The corresponding output string.
    pub fn add_form(&mut self, move_str: &str, output: &str) {
        self.forms.insert(move_str.to_string(), output.to_string());
    }

    /// Generates a string for a single move.
    /// # Arguments
    /// * `move` - The move to generate.
    fn generate_move(&self, move_str: &str) -> String {
        self.forms.get(move_str).cloned().unwrap_or_else(|| move_str.to_string())
    }

    /// Joins phrases into a single string with punctuation.
    /// # Arguments
    /// * `phrases` - The phrases to join.
    fn join_phrases(&self, phrases: &[String]) -> String {
        let mut result = String::new();
        for p in phrases {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(p);
            if !p.ends_with('.') && !p.ends_with('?') && !p.ends_with('!') {
                result.push('.');
            }
        }
        result
    }
}

/// Implements the Grammar trait for SimpleGenGrammar.
impl Grammar for SimpleGenGrammar {
    fn generate(&self, moves: &TSet<String>) -> String {
        let phrases: Vec<String> = moves.elements.iter().map(|m| self.generate_move(m)).collect();
        self.join_phrases(&phrases)
    }

    fn interpret(&self, input: &str) -> Option<TSet<String>> {
        let mut moves = TSet::new();
        
        // Handle special cases first
        if input == "quit" || input == "exit" {
            moves.add("Quit()".to_string()).ok();
        }
        // Try to parse as a question
        else if let Ok(_question) = Question::new(input) {
            moves.add(format!("Ask('{}')", input)).ok();
        }
        // Try to parse as an answer
        else if let Ok(_answer) = Ans::new(input) {
            moves.add(format!("Answer({})", input)).ok();
        }
        else {
            return None;
        }
        
        Some(moves)
    }

}


/// CFG Grammar Rule structure for parsing context-free grammar files
#[derive(Debug, Clone)]
struct CFGRule {
    lhs: String,           // Left-hand side (e.g., "USR[sem=?s]")
    rhs: Vec<String>,      // Right-hand side alternatives (e.g., ["ANSWER[sem=?s]", "ASK[sem=?s]"])
    features: HashMap<String, String>, // Feature annotations (e.g., sem=?s, q=?q)
}

/// CFG Grammar structure for parsing travel.fcfg files
struct CFGGrammar {
    rules: Vec<CFGRule>,
    terminals: HashMap<String, Vec<String>>, // Terminal mappings (e.g., 'price' -> WHQ[q=price])
}

impl CFGGrammar {
    /// Creates a new empty CFG grammar
    fn new() -> Self {
        CFGGrammar {
            rules: Vec::new(),
            terminals: HashMap::new(),
        }
    }

    /// Loads CFG rules from a file (basic implementation)
    fn load_from_file(&mut self, _filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder: In full implementation, this would parse travel.fcfg
        // For now, add some basic rules based on travel.fcfg
        
        // Add basic rule: USR[sem=?s] -> ANSWER[sem=?s] | ASK[sem=?s]
        self.rules.push(CFGRule {
            lhs: "USR[sem=?s]".to_string(),
            rhs: vec!["ANSWER[sem=?s]".to_string(), "ASK[sem=?s]".to_string()],
            features: HashMap::from([("sem".to_string(), "?s".to_string())]),
        });

        // Add terminal mappings
        self.terminals.insert("price".to_string(), vec!["WHQ[q=price]".to_string()]);
        self.terminals.insert("plane".to_string(), vec!["CAT[cat=how, ind=plane]".to_string()]);
        self.terminals.insert("train".to_string(), vec!["CAT[cat=how, ind=train]".to_string()]);
        
        Ok(())
    }

    /// Basic parsing of input using CFG rules (placeholder)
    fn parse(&self, input: &str) -> Option<String> {
        // Placeholder: Check terminals first
        if let Some(categories) = self.terminals.get(input) {
            return categories.first().cloned();
        }
        None
    }
}

// Database

/// Trait for consulting a database with questions.
trait Database {
    /// Consults the database with a question and context.
    /// # Arguments
    /// * `question` - The question to consult.
    /// * `context` - The context propositions.
    fn consult_db(&self, question: &Question, context: &TSet<Prop>) -> Prop;
}

/// A travel database storing entries as key-value maps.
pub struct TravelDB {
    entries: Vec<HashMap<String, String>>, // Database entries
}

/// Implementation of methods for the TravelDB struct.
impl TravelDB {
    /// Creates a new empty TravelDB.
    pub fn new() -> Self {
        TravelDB { entries: Vec::new() }
    }

    /// Adds an entry to the database.
    /// # Arguments
    /// * `entry` - The key-value map to add.
    pub fn add_entry(&mut self, entry: HashMap<String, String>) {
        self.entries.push(entry);
    }

    /// Retrieves a context value for a predicate.
    /// # Arguments
    /// * `context` - The context propositions.
    /// * `pred` - The predicate to look up.
    fn get_context(&self, context: &TSet<Prop>, pred: &str) -> Option<String> {
        for prop in &context.elements {
            if prop.pred.0.content == pred {
                return prop.ind.as_ref().map(|ind| ind.0.content.clone());
            }
        }
        None
    }

    /// Looks up an entry by departure city, destination city, and day.
    /// # Arguments
    /// * `depart_city` - Departure city.
    /// * `dest_city` - Destination city.
    /// * `day` - Departure day.
    fn lookup_entry(&self, depart_city: &str, dest_city: &str, day: &str) -> Option<&HashMap<String, String>> {
        for entry in &self.entries {
            if entry.get("from") == Some(&depart_city.to_string())
                && entry.get("to") == Some(&dest_city.to_string())
                && entry.get("day") == Some(&day.to_string())
            {
                return Some(entry);
            }
        }
        None
    }
}

/// Implements the Database trait for TravelDB.
impl Database for TravelDB {
    fn consult_db(&self, question: &Question, context: &TSet<Prop>) -> Prop {
        let depart_city = self.get_context(context, "depart_city").unwrap_or_default();
        let dest_city = self.get_context(context, "dest_city").unwrap_or_default();
        let day = self.get_context(context, "depart_day").unwrap_or_default();
        let entry = self.lookup_entry(&depart_city, &dest_city, &day).expect("Entry not found");
        let price = entry.get("price").expect("Price not found");
        Prop {
            pred: Pred0::new("price").unwrap(),
            ind: Some(Ind::new(price).unwrap()),
            yes: true,
        }
    }
}

// Domain

/// Represents the domain knowledge, including predicates, sorts, and plans.
pub struct Domain {
    preds0: HashSet<String>, // Zero-place predicates
    preds1: HashMap<String, String>, // One-place predicates with their sorts
    sorts: HashMap<String, HashSet<String>>, // Sorts and their individuals
    inds: HashMap<String, String>, // Individuals and their sorts
    plans: HashMap<String, Vec<String>>, // Question-triggered plans
}

/// Implementation of methods for the Domain struct.
impl Domain {
    /// Creates a new Domain.
    /// # Arguments
    /// * `preds0` - Zero-place predicates.
    /// * `preds1` - One-place predicates with their sorts.
    /// * `sorts` - Sorts and their individuals.
    pub fn new(
        preds0: HashSet<String>,
        preds1: HashMap<String, String>,
        sorts: HashMap<String, HashSet<String>>,
    ) -> Self {
        let inds = sorts
            .iter()
            .flat_map(|(sort, inds)| inds.iter().map(move |ind| (ind.clone(), sort.clone())))
            .collect();
        Domain {
            preds0,
            preds1,
            sorts,
            inds,
            plans: HashMap::new(),
        }
    }

    /// Adds a plan for a question.
    /// # Arguments
    /// * `trigger` - The question that triggers the plan.
    /// * `plan` - The plan constructors to execute.
    pub fn add_plan(&mut self, trigger: Question, plan: Vec<String>) {
        self.plans.insert(trigger.to_string(), plan);
    }

    /// Checks if an answer is relevant to a question.
    /// # Arguments
    /// * `answer` - The answer to check.
    /// * `question` - The question to check against.
    fn relevant(&self, answer: &Ans, question: &Question) -> bool {
        match (answer, question) {
            (Ans::Prop(prop), Question::WhQ(whq)) => prop.pred.0.content == whq.pred.0.content,
            (Ans::ShortAns(short), Question::WhQ(whq)) => {
                let sort1 = self.inds.get(&short.ind.0.content);
                let sort2 = self.preds1.get(&whq.pred.0.content);
                sort1.is_some() && sort2.is_some() && sort1 == sort2
            }
            (Ans::YesNo(_), Question::YNQ(_)) => true,
            (Ans::Prop(prop), Question::YNQ(ynq)) => prop == &ynq.prop,
            (Ans::Prop(prop), Question::AltQ(altq)) => {
                altq.ynqs.iter().any(|ynq| prop == &ynq.prop)
            }
            (Ans::YesNo(_), Question::AltQ(_)) => true,
            _ => false,
        }
    }

    /// Checks if an answer resolves a question.
    /// # Arguments
    /// * `answer` - The answer to check.
    /// * `question` - The question to check against.
    fn resolves(&self, answer: &Ans, question: &Question) -> bool {
        if self.relevant(answer, question) {
            match (answer, question) {
                (Ans::YesNo(_), Question::YNQ(_)) => true,
                (Ans::ShortAns(short), Question::WhQ(_)) => short.yes,
                (Ans::Prop(prop), Question::WhQ(_)) => prop.yes,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Combines a question and answer into a proposition.
    /// # Arguments
    /// * `question` - The question.
    /// * `answer` - The answer.
    fn combine(&self, question: &Question, answer: &Ans) -> Result<Prop, Box<dyn std::error::Error>> {
        assert!(self.relevant(answer, question));
        match (question, answer) {
            (Question::WhQ(whq), Ans::ShortAns(short)) => {
                let mut prop = whq.pred.apply(&short.ind)?;
                if !short.yes {
                    prop.yes = false;
                }
                Ok(prop)
            }
            (Question::YNQ(ynq), Ans::YesNo(yesno)) => {
                let mut prop = ynq.prop.clone();
                if prop.yes != yesno.yes {
                    prop.yes = !prop.yes;
                }
                Ok(prop)
            }
            _ => match answer {
                Ans::Prop(p) => Ok(p.clone()),
                _ => panic!("Invalid combination"),
            },
        }
    }

    /// Retrieves the plan for a question.
    /// # Arguments
    /// * `question` - The question to get the plan for.
    fn get_plan(&self, question: &Question) -> Option<Stack<String>> {
        self.plans.get(&question.to_string()).map(|plan| {
            let mut stack = Stack::new();
            for construct in plan.iter().rev() {
                stack.push(construct.clone()).unwrap();
            }
            stack
        })
    }
}

// IBIS Information State

/// Represents the Information-Based Inquiry System (IBIS) information state.
struct IBISInfostate {
    is: Record, // The record storing private and shared state
}

/// Implementation of methods for the IBISInfostate struct.
impl IBISInfostate {
    /// Initializes the information state with default fields.
    fn init_is(&mut self) {
        let mut fields = HashMap::new();
        fields.insert("agenda".to_string(), Box::new(Stack::<String>::new()) as Box<dyn Any>);
        fields.insert("plan".to_string(), Box::new(Stack::<String>::new()) as Box<dyn Any>);
        fields.insert("bel".to_string(), Box::new(TSet::<String>::new()) as Box<dyn Any>);
        fields.insert("com".to_string(), Box::new(TSet::<String>::new()) as Box<dyn Any>);
        fields.insert("qud".to_string(), Box::new(StackSet::<String>::new()) as Box<dyn Any>);
        self.is = Record::new(fields);
    }

    /// Prints the information state with a prefix.
    /// # Arguments
    /// * `prefix` - The prefix for each line.
    fn print_is(&self, prefix: &str) {
        println!("{}", self.is.pformat(prefix, "    "));
    }
}

// IBIS Controller

/// Controls the IBIS dialogue system.
pub struct IBISController {
    is: IBISInfostate, // Information state
    mivs: StandardMIVS, // Minimal information state
    domain: Domain, // Domain knowledge
    database: TravelDB, // Travel database
    grammar: SimpleGenGrammar, // Grammar for generation and interpretation
    input_handler: Box<dyn InputHandler>, // Input handling abstraction
}

/// Implementation of methods for the IBISController struct.
impl IBISController {
    /// Creates a new IBISController.
    /// # Arguments
    /// * `domain` - The domain knowledge.
    /// * `database` - The travel database.
    /// * `grammar` - The grammar for dialogue.
    pub fn new(domain: Domain, database: TravelDB, grammar: SimpleGenGrammar) -> Self {
        Self::with_input_handler(domain, database, grammar, Box::new(StandardInputHandler))
    }
    
    pub fn with_input_handler(domain: Domain, database: TravelDB, grammar: SimpleGenGrammar, input_handler: Box<dyn InputHandler>) -> Self {
        IBISController {
            is: IBISInfostate { is: Record::new(HashMap::new()) },
            mivs: StandardMIVS {
                input: Value::new_type(|_: &String| true),
                latest_speaker: Value::new_allowed(HashSet::from([Speaker::USR, Speaker::SYS])),
                latest_moves: TSet::new(),
                next_moves: Stack::new(),
                output: Value::new_type(|_: &String| true),
                program_state: Value::new_allowed(HashSet::from([ProgramState::RUN, ProgramState::QUIT])),
            },
            domain,
            database,
            grammar,
            input_handler,
        }
    }

    /// Selects the next moves (placeholder).
    fn select(&mut self) {
        // Placeholder: Implement selection logic
    }

    /// Generates output from the next moves.
    fn generate(&mut self) {
        // Convert stack to TSet for generation
        let mut moves_set = TSet::new();
        for element in &self.mivs.next_moves.elements {
            moves_set.add(element.clone()).ok();
        }
        let output = self.grammar.generate(&moves_set);
        self.mivs.output.set(output).unwrap();
    }

    /// Outputs the generated response.
    fn output(&mut self) {
        println!("S> {}", self.mivs.output.get().unwrap_or(&"[---]".to_string()));
        println!();
        self.mivs.latest_speaker.set(Speaker::SYS).unwrap();
        self.mivs.latest_moves.clear();
        for element in &self.mivs.next_moves.elements {
            self.mivs.latest_moves.add(element.clone()).ok();
        }
        self.mivs.next_moves.clear();
    }

    /// Reads user input.
    fn input(&mut self) {
        if let Some(input) = self.input_handler.read_line() {
            self.mivs.input.set(input).unwrap();
            self.mivs.latest_speaker.set(Speaker::USR).unwrap();
        } else {
            self.mivs.program_state.set(ProgramState::QUIT).unwrap();
        }
    }

    /// Interprets the user input into moves.
    fn interpret(&mut self) {
        self.mivs.latest_moves.clear();
        if let Some(input) = self.mivs.input.get() {
            if !input.is_empty() {
                if let Some(moves) = self.grammar.interpret(input) {
                    for move_str in &moves.elements {
                        self.mivs.latest_moves.add(move_str.clone()).ok();
                    }
                } else {
                    println!("Did not understand: {}", input);
                }
            }
        }
    }

    /// Updates the dialogue state (placeholder).
    fn update(&mut self) {
        // Placeholder: Implement update logic
    }
}

/// Implements the DialogueManager trait for IBISController.
impl DialogueManager for IBISController {
    fn reset(&mut self) {
        self.is.init_is();
        self.mivs.init_mivs();
    }

    fn control(&mut self) {
        self.mivs.next_moves.push("Greet()".to_string()).unwrap();
        self.print_state();
        while self.mivs.program_state.get() != Some(&ProgramState::QUIT) {
            self.select();
            if !self.mivs.next_moves.elements.is_empty() {
                self.generate();
                self.output();
                self.update();
                self.print_state();
            }
            self.input();
            self.interpret();
            self.update();
            self.print_state();
        }
    }

    fn print_state(&self) {
        println!("+------------------------ - -  -");
        self.mivs.print_mivs("| ");
        println!("|");
        self.is.print_is("| ");
        println!("+------------------------ - -  -");
        println!();
    }
}

/// Additional implementation to make IBISController usable
impl IBISController {
    /// Runs the dialogue manager (public interface)
    pub fn run(&mut self) {
        <Self as DialogueManager>::run(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for core data structures
    #[test]
    fn test_value_creation_and_operations() {
        let allowed_values = HashSet::from(["a".to_string(), "b".to_string(), "c".to_string()]);
        let mut value = Value::new_allowed(allowed_values);
        
        // Test setting valid value
        assert!(value.set("a".to_string()).is_ok());
        assert_eq!(value.get(), Some(&"a".to_string()));
        
        // Test setting invalid value
        assert!(value.set("d".to_string()).is_err());
        
        // Test clearing value
        value.clear();
        assert_eq!(value.get(), None);
    }
    
    #[test]
    fn test_value_with_type_constraint() {
        let mut value = Value::new_type(|s: &String| s.len() > 2);
        
        assert!(value.set("hello".to_string()).is_ok());
        assert!(value.set("hi".to_string()).is_err());
        assert_eq!(value.get(), Some(&"hello".to_string()));
    }
    
    #[test]
    fn test_stack_operations() {
        let mut stack = Stack::new();
        
        // Test empty stack
        assert!(stack.top().is_err());
        assert!(stack.pop().is_err());
        assert_eq!(stack.len(), 0);
        
        // Test push and basic operations
        assert!(stack.push("item1".to_string()).is_ok());
        assert!(stack.push("item2".to_string()).is_ok());
        assert_eq!(stack.len(), 2);
        
        // Test top and pop
        assert_eq!(stack.top().unwrap(), &"item2".to_string());
        assert_eq!(stack.pop().unwrap(), "item2".to_string());
        assert_eq!(stack.len(), 1);
        
        // Test clear
        stack.clear();
        assert_eq!(stack.len(), 0);
    }
    
    #[test]
    fn test_stack_with_type_constraint() {
        let mut stack = Stack::with_type(|s: &String| s.starts_with("valid_"));
        
        assert!(stack.push("valid_item".to_string()).is_ok());
        assert!(stack.push("invalid_item".to_string()).is_err());
        assert_eq!(stack.len(), 1);
    }
    
    #[test]
    fn test_stackset_operations() {
        let mut stackset = StackSet::new();
        
        // Test push with uniqueness
        assert!(stackset.push("item1".to_string()).is_ok());
        assert!(stackset.push("item2".to_string()).is_ok());
        assert!(stackset.push("item1".to_string()).is_ok()); // Should move to top
        
        assert!(stackset.contains(&"item1".to_string()));
        assert!(stackset.contains(&"item2".to_string()));
        assert!(!stackset.contains(&"item3".to_string()));
    }
    
    #[test]
    fn test_tset_operations() {
        let mut tset = TSet::new();
        
        assert!(tset.add("item1".to_string()).is_ok());
        assert!(tset.add("item2".to_string()).is_ok());
        assert!(tset.add("item1".to_string()).is_ok()); // Duplicate should be fine
        
        assert_eq!(tset.len(), 2); // Sets contain unique elements
        assert!(tset.contains(&"item1".to_string()));
        assert!(tset.contains(&"item2".to_string()));
        
        tset.clear();
        assert_eq!(tset.len(), 0);
    }
    
    #[test]
    fn test_tset_with_type_constraint() {
        let mut tset = TSet::with_type(|s: &String| s.len() <= 5);
        
        assert!(tset.add("short".to_string()).is_ok());
        assert!(tset.add("toolongstring".to_string()).is_err());
        assert_eq!(tset.len(), 1);
    }
    
    // Tests for semantic types
    #[test]
    fn test_atomic_creation() {
        // Valid atomic values
        assert!(Atomic::new("hello").is_ok());
        assert!(Atomic::new("test_atom").is_ok());
        assert!(Atomic::new("atom-with-dash").is_ok());
        assert!(Atomic::new("atom+with+plus").is_ok());
        assert!(Atomic::new("atom:with:colon").is_ok());
        
        // Invalid atomic values
        assert!(Atomic::new("").is_err()); // Empty
        assert!(Atomic::new("yes").is_err()); // Reserved word
        assert!(Atomic::new("no").is_err()); // Reserved word
        assert!(Atomic::new("123invalid").is_err()); // Starts with number
        assert!(Atomic::new("invalid@char").is_err()); // Invalid character
    }
    
    #[test]
    fn test_ind_creation_and_display() {
        let ind = Ind::new("paris").unwrap();
        assert_eq!(ind.to_string(), "paris");
        
        assert!(Ind::new("").is_err());
        assert!(Ind::new("invalid@").is_err());
    }
    
    #[test]
    fn test_pred0_creation_and_display() {
        let pred = Pred0::new("expensive").unwrap();
        assert_eq!(pred.to_string(), "expensive");
        
        assert!(Pred0::new("").is_err());
        assert!(Pred0::new("yes").is_err());
    }
    
    #[test]
    fn test_pred1_creation_and_apply() {
        let pred1 = Pred1::new("city").unwrap();
        let ind = Ind::new("paris").unwrap();
        
        let prop = pred1.apply(&ind).unwrap();
        assert_eq!(prop.to_string(), "city(paris)");
        assert!(prop.yes);
        
        assert!(Pred1::new("").is_err());
    }
    
    #[test]
    fn test_sort_creation() {
        let sort = Sort::new("city").unwrap();
        assert_eq!(sort.to_string(), "city");
        
        assert!(Sort::new("").is_err());
    }
    
    #[test]
    fn test_prop_creation_and_parsing() {
        // Test basic proposition
        let prop = Prop::new("expensive").unwrap();
        assert_eq!(prop.to_string(), "expensive()");
        assert!(prop.yes);
        
        // Test negated proposition
        let prop = Prop::new("-expensive").unwrap();
        assert_eq!(prop.to_string(), "-expensive()");
        assert!(!prop.yes);
        
        // Test proposition with individual
        let prop = Prop::new("city(paris)").unwrap();
        assert_eq!(prop.to_string(), "city(paris)");
        assert!(prop.yes);
        assert_eq!(prop.ind.as_ref().unwrap().to_string(), "paris");
    }
    
    #[test]
    fn test_shortans_creation_and_parsing() {
        // Positive short answer
        let ans = ShortAns::new("paris").unwrap();
        assert_eq!(ans.to_string(), "paris");
        assert!(ans.yes);
        
        // Negative short answer  
        let ans = ShortAns::new("-paris").unwrap();
        assert_eq!(ans.to_string(), "-paris");
        assert!(!ans.yes);
        
        assert!(ShortAns::new("").is_err());
    }
    
    #[test]
    fn test_yesno_creation() {
        let yes_ans = YesNo::new("yes").unwrap();
        assert_eq!(yes_ans.to_string(), "yes");
        assert!(yes_ans.yes);
        
        let no_ans = YesNo::new("no").unwrap();
        assert_eq!(no_ans.to_string(), "no");
        assert!(!no_ans.yes);
        
        assert!(YesNo::new("maybe").is_err());
    }
    
    #[test]
    fn test_ans_enum_parsing() {
        // Test yes/no parsing
        let ans = Ans::new("yes").unwrap();
        match ans {
            Ans::YesNo(yesno) => assert!(yesno.yes),
            _ => panic!("Expected YesNo variant"),
        }
        
        // Test short answer parsing
        let ans = Ans::new("paris").unwrap();
        match ans {
            Ans::ShortAns(short) => {
                assert_eq!(short.to_string(), "paris");
                assert!(short.yes);
            },
            _ => panic!("Expected ShortAns variant"),
        }
        
        // Test proposition parsing
        let ans = Ans::new("city(paris)").unwrap();
        match ans {
            Ans::Prop(prop) => {
                assert_eq!(prop.to_string(), "city(paris)");
                assert!(prop.yes);
            },
            _ => panic!("Expected Prop variant"),
        }
        
        assert!(Ans::new("invalid(syntax").is_err());
    }
    
    #[test] 
    fn test_whq_creation_and_parsing() {
        // Test standard wh-question format
        let whq = WhQ::new("?x.city(x)").unwrap();
        assert_eq!(whq.pred.to_string(), "city");
        
        // Test simplified format
        let whq = WhQ::new("city").unwrap();
        assert_eq!(whq.pred.to_string(), "city");
        
        assert!(WhQ::new("").is_err());
    }
    
    #[test]
    fn test_ynq_creation_and_parsing() {
        let ynq = YNQ::new("?expensive").unwrap();
        assert_eq!(ynq.prop.to_string(), "expensive()");
        
        let ynq = YNQ::new("expensive").unwrap();
        assert_eq!(ynq.prop.to_string(), "expensive()");
        
        assert!(YNQ::new("").is_err());
    }
    
    #[test]
    fn test_question_enum_parsing() {
        // Test wh-question parsing
        let q = Question::new("?x.city(x)").unwrap();
        match q {
            Question::WhQ(whq) => assert_eq!(whq.pred.to_string(), "city"),
            _ => panic!("Expected WhQ variant"),
        }
        
        // Test yes/no question parsing
        let q = Question::new("?expensive").unwrap();
        match q {
            Question::YNQ(ynq) => assert_eq!(ynq.prop.pred.to_string(), "expensive"),
            _ => panic!("Expected YNQ variant"),
        }
        
        assert!(Question::new("invalid").is_err());
    }
    
    // Tests for dialogue components
    #[test]
    fn test_dialogue_moves() {
        // Test Greet
        let greet = Greet;
        assert_eq!(greet.to_string(), "Greet()");
        
        // Test Quit
        let quit = Quit;
        assert_eq!(quit.to_string(), "Quit()");
        
        // Test Ask
        let question = Question::new("?expensive").unwrap();
        let ask = Ask::new(question);
        assert!(ask.to_string().contains("Ask"));
        assert!(ask.to_string().contains("expensive"));
        
        // Test Answer
        let answer_content = Ans::new("yes").unwrap();
        let answer = Answer::new(answer_content);
        assert!(answer.to_string().contains("Answer"));
        assert!(answer.to_string().contains("yes"));
    }
    
    #[test]
    fn test_icm_creation() {
        let icm = ICM::new("per", "pos", Some("understood".to_string()));
        assert_eq!(icm.to_string(), "icm:per*pos:'understood'");
        
        let icm_no_content = ICM::new("sem", "neg", None);
        assert_eq!(icm_no_content.to_string(), "icm:sem*neg");
    }
    
    #[test]
    fn test_plan_constructors() {
        let question = Question::new("?expensive").unwrap();
        
        // Test Respond
        let respond = Respond::new(question.clone());
        assert!(respond.to_string().contains("Respond"));
        assert!(respond.to_string().contains("expensive"));
        
        // Test ConsultDB
        let consult = ConsultDB::new(question.clone());
        assert!(consult.to_string().contains("ConsultDB"));
        assert!(consult.to_string().contains("expensive"));
        
        // Test Findout
        let findout = Findout::new(question.clone());
        assert!(findout.to_string().contains("Findout"));
        assert!(findout.to_string().contains("expensive"));
        
        // Test Raise
        let raise = Raise::new(question.clone());
        assert!(raise.to_string().contains("Raise"));
        assert!(raise.to_string().contains("expensive"));
        
        // Test If
        let if_plan = If::new(
            question, 
            vec!["ConsultDB".to_string()], 
            vec!["Greet".to_string()]
        );
        assert!(if_plan.to_string().contains("If"));
    }
    
    // Tests for grammar functionality
    #[test]
    fn test_simple_gen_grammar() {
        let mut grammar = SimpleGenGrammar::new();
        
        // Test adding custom forms
        grammar.add_form("Ask('?price')", "What is the price?");
        grammar.add_form("Answer(paris)", "The answer is Paris.");
        
        // Test generation
        let mut moves = TSet::new();
        moves.add("Greet()".to_string()).unwrap();
        let output = grammar.generate(&moves);
        assert_eq!(output, "Hello.");
        
        // Test interpretation - "quit" is handled as special case in the grammar
        let interpreted = grammar.interpret("quit");
        assert!(interpreted.is_some());
        let moves = interpreted.unwrap();
        assert!(moves.elements.iter().any(|m| m.contains("Quit")));
        
        // Test question interpretation  
        let interpreted = grammar.interpret("?expensive");
        assert!(interpreted.is_some());
        let moves = interpreted.unwrap();
        assert!(moves.elements.iter().any(|m| m.contains("Ask") && m.contains("expensive")));
        
        // Test answer interpretation
        let interpreted = grammar.interpret("yes");
        assert!(interpreted.is_some());
        let moves = interpreted.unwrap();
        assert!(moves.elements.iter().any(|m| m.contains("Answer") && m.contains("yes")));
        
        // Test unrecognized input
        let interpreted = grammar.interpret("random gibberish");
        assert!(interpreted.is_none());
    }
    
    // Tests for database functionality
    #[test]
    fn test_travel_db() {
        let mut db = TravelDB::new();
        
        // Add sample entries
        let mut entry1 = HashMap::new();
        entry1.insert("from".to_string(), "paris".to_string());
        entry1.insert("to".to_string(), "london".to_string());
        entry1.insert("day".to_string(), "monday".to_string());
        entry1.insert("price".to_string(), "200".to_string());
        db.add_entry(entry1);
        
        let mut entry2 = HashMap::new();
        entry2.insert("from".to_string(), "london".to_string());
        entry2.insert("to".to_string(), "paris".to_string());
        entry2.insert("day".to_string(), "tuesday".to_string());
        entry2.insert("price".to_string(), "180".to_string());
        db.add_entry(entry2);
        
        // Test lookup
        let result = db.lookup_entry("paris", "london", "monday");
        assert!(result.is_some());
        assert_eq!(result.unwrap().get("price"), Some(&"200".to_string()));
        
        let no_result = db.lookup_entry("invalid", "route", "never");
        assert!(no_result.is_none());
        
        // Test context retrieval (using mock context)
        let mut context = TSet::new();
        let prop1 = Prop {
            pred: Pred0::new("depart_city").unwrap(),
            ind: Some(Ind::new("paris").unwrap()),
            yes: true,
        };
        context.add(prop1).unwrap();
        
        let context_value = db.get_context(&context, "depart_city");
        assert_eq!(context_value, Some("paris".to_string()));
        
        let no_context = db.get_context(&context, "nonexistent");
        assert_eq!(no_context, None);
    }
    
    // Tests for domain functionality
    #[test]
    fn test_domain_creation_and_operations() {
        let preds0 = HashSet::from(["expensive".to_string(), "available".to_string()]);
        let mut preds1 = HashMap::new();
        preds1.insert("city".to_string(), "location".to_string());
        preds1.insert("transport".to_string(), "vehicle".to_string());
        
        let mut sorts = HashMap::new();
        sorts.insert("location".to_string(), HashSet::from(["paris".to_string(), "london".to_string()]));
        sorts.insert("vehicle".to_string(), HashSet::from(["plane".to_string(), "train".to_string()]));
        
        let mut domain = Domain::new(preds0, preds1, sorts);
        
        // Test individuals are correctly inferred
        assert_eq!(domain.inds.get("paris"), Some(&"location".to_string()));
        assert_eq!(domain.inds.get("plane"), Some(&"vehicle".to_string()));
        
        // Test adding plans
        let question = Question::new("?expensive").unwrap();
        let plan = vec!["ConsultDB".to_string(), "Respond".to_string()];
        domain.add_plan(question.clone(), plan);
        
        let retrieved_plan = domain.get_plan(&question);
        assert!(retrieved_plan.is_some());
        let plan_stack = retrieved_plan.unwrap();
        assert_eq!(plan_stack.len(), 2);
        
        // Test relevance checking
        let ans_yes = Ans::new("yes").unwrap();
        let ynq = Question::new("?expensive").unwrap();
        assert!(domain.relevant(&ans_yes, &ynq));
        
        let ans_paris = Ans::new("paris").unwrap();
        let whq = Question::new("?x.city(x)").unwrap();
        assert!(domain.relevant(&ans_paris, &whq));
        
        // Test resolution checking
        let ans_yes = Ans::new("yes").unwrap();
        let ynq = Question::new("?expensive").unwrap();
        assert!(domain.resolves(&ans_yes, &ynq));
        
        let ans_no = Ans::new("no").unwrap();
        assert!(domain.resolves(&ans_no, &ynq));
        
        // Test combination
        let combined = domain.combine(&whq, &ans_paris);
        assert!(combined.is_ok());
        let prop = combined.unwrap();
        assert_eq!(prop.pred.to_string(), "city");
        assert_eq!(prop.ind.as_ref().unwrap().to_string(), "paris");
    }
    
    // Test for enums
    #[test]
    fn test_speaker_enum() {
        let usr = Speaker::new("USR").unwrap();
        assert_eq!(usr.to_string(), "USR");
        
        let sys = Speaker::new("SYS").unwrap();  
        assert_eq!(sys.to_string(), "SYS");
        
        assert!(Speaker::new("INVALID").is_none());
    }
    
    #[test]
    fn test_program_state_enum() {
        let run = ProgramState::new("RUN").unwrap();
        assert_eq!(run.to_string(), "RUN");
        
        let quit = ProgramState::new("QUIT").unwrap();
        assert_eq!(quit.to_string(), "QUIT");
        
        assert!(ProgramState::new("INVALID").is_none());
    }
    
    // Tests for input handlers
    #[test]
    fn test_demo_input_handler() {
        let inputs = vec!["hello".to_string(), "?expensive".to_string(), "quit".to_string()];
        let mut handler = DemoInputHandler::new(inputs);
        
        assert!(handler.has_input());
        assert_eq!(handler.read_line(), Some("hello".to_string()));
        
        assert!(handler.has_input());
        assert_eq!(handler.read_line(), Some("?expensive".to_string()));
        
        assert!(handler.has_input());
        assert_eq!(handler.read_line(), Some("quit".to_string()));
        
        assert!(!handler.has_input());
        assert_eq!(handler.read_line(), None);
    }
    
    // Integration test for IBISController
    #[test]
    fn test_ibis_controller_creation() {
        // Create domain
        let preds0 = HashSet::from(["expensive".to_string()]);
        let preds1 = HashMap::from([("city".to_string(), "location".to_string())]);
        let sorts = HashMap::from([("location".to_string(), HashSet::from(["paris".to_string()]))]);
        let domain = Domain::new(preds0, preds1, sorts);
        
        // Create database
        let database = TravelDB::new();
        
        // Create grammar
        let grammar = SimpleGenGrammar::new();
        
        // Create demo input handler
        let inputs = vec!["hello".to_string(), "quit".to_string()];
        let input_handler = Box::new(DemoInputHandler::new(inputs));
        
        // Create controller
        let controller = IBISController::with_input_handler(domain, database, grammar, input_handler);
        
        // Basic assertion that controller was created successfully
        assert!(matches!(controller.mivs.program_state.get(), None)); // Initially unset
    }
}