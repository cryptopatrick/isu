//! isu is a Rust implementation of Information State Update theory.
//! The library can be use for Issue-Based Dialogue Management and 
//! Conversational Agent Architecture.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{self, Write};
use std::rc::Rc;

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
#[derive(Clone)]
struct Value<T: Clone> {
    value: Option<T>, // The stored value, if any
    allowed_values: HashSet<T>, // Set of permitted values
    type_constraint: Option<fn(&T) -> bool>, // Optional type checking function
}

/// Implementation of methods for the Value struct.
impl<T: Clone + PartialEq + fmt::Display> Value<T> {
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
            type_constraint: Some(Box::new(type_check) as fn(&T) -> bool),
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
        if let Some(check) = self.type_constraint {
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
impl<T: Clone + fmt::Display> fmt::Display for Value<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.value {
            Some(v) => write!(f, "<{}>", v),
            None => write!(f, "<>"),
        }
    }
}

// Record struct

/// A key-value store with type checking for fields.
#[derive(Clone)]
struct Record {
    typedict: HashMap<String, fn(&dyn std::any::Any) -> bool>, // Type checking functions for fields
    fields: HashMap<String, Box<dyn std::any::Any>>, // Stored field values
}

/// Implementation of methods for the Record struct.
impl Record {
    /// Creates a new Record with initial fields and inferred type checks.
    /// # Arguments
    /// * `fields` - Initial key-value pairs.
    fn new(fields: HashMap<String, Box<dyn std::any::Any>>) -> Self {
        let mut typedict = HashMap::new();
        for (key, value) in &fields {
            let type_id = value.type_id();
            typedict.insert(key.clone(), move |v: &dyn std::any::Any| v.type_id() == type_id);
        }
        Record { typedict, fields }
    }

    /// Returns a HashMap of field keys to their values.
    fn as_dict(&self) -> HashMap<String, &dyn std::any::Any> {
        self.fields.iter().map(|(k, v)| (k.clone(), v.as_ref())).collect()
    }

    /// Checks if a value matches the expected type for a given key.
    /// # Arguments
    /// * `key` - The field key to check.
    /// * `value` - Optional value to type check.
    fn typecheck(&self, key: &str, value: Option<&dyn std::any::Any>) -> Result<(), String> {
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
    fn get(&self, key: &str) -> Option<&dyn std::any::Any> {
        self.typecheck(key, None).ok()?;
        self.fields.get(key).map(|v| v.as_ref())
    }

    /// Sets a field value after type checking.
    /// # Arguments
    /// * `key` - The field key.
    /// * `value` - The value to set.
    fn set(&mut self, key: &str, value: Box<dyn std::any::Any>) -> Result<(), String> {
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
            result.push_str(key);
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
#[derive(Clone)]
struct Stack<T: Clone> {
    elements: Vec<T>, // The stack's elements
    type_constraint: Option<fn(&T) -> bool>, // Optional type checking function
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
            type_constraint: Some(Box::new(type_check) as fn(&T) -> bool),
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
        if let Some(check) = self.type_constraint {
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
struct StackSet<T: Clone + PartialEq> {
    stack: Stack<T>, // Underlying stack for storage
}

/// Implementation of methods for the StackSet struct.
impl<T: Clone + PartialEq + fmt::Display> StackSet<T> {
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
impl<T: Clone + PartialEq + fmt::Display> fmt::Display for StackSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{{ {} <}}", self.stack.to_string())
    }
}

// TSet struct

/// A typed set with optional type constraints for elements.
#[derive(Clone)]
struct TSet<T: Clone + PartialEq> {
    elements: HashSet<T>, // The set of elements
    type_constraint: Option<fn(&T) -> bool>, // Optional type checking function
}

/// Implementation of methods for the TSet struct.
impl<T: Clone + PartialEq + fmt::Display> TSet<T> {
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
            type_constraint: Some(Box::new(type_check) as fn(&T) -> bool),
        }
    }

    /// Adds an element to the TSet after type checking.
    /// # Arguments
    /// * `value` - The value to add.
    fn add(&mut self, value: T) -> Result<(), String> {
        if let Some(check) = self.type_constraint {
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
impl<T: Clone + PartialEq + fmt::Display> fmt::Display for TSet<T> {
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
        #[derive(Clone, PartialEq, Debug)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
            (false, &s[1..], None)
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
enum Question {
    WhQ(WhQ), // Wh-question
    YNQ(YNQ), // Yes/no question
    AltQ(AltQ), // Alternative question
}

/// Implementation of methods for the Question enum.
impl Question {
    /// Creates a new Question from a string.
    /// # Arguments
    /// * `s` - The string to parse.
    fn new(s: &str) -> Result<Self, String> {
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
struct ConsultDB {
    content: Question, // The question to consult
}

/// Implementation of methods for the ConsultDB struct.
impl ConsultDB {
    /// Creates a new ConsultDB plan.
    /// # Arguments
    /// * `content` - The question to consult.
    fn new(content: Question) -> Self {
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
struct Findout {
    content: Question, // The question to find out
}

/// Implementation of methods for the Findout struct.
impl Findout {
    /// Creates a new Findout plan.
    /// # Arguments
    /// * `content` - The question to find out.
    fn new(content: Question) -> Self {
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
struct If {
    cond: Question, // The condition question
    iftrue: Vec<Box<dyn PlanConstructor>>, // Plans if condition is true
    iffalse: Vec<Box<dyn PlanConstructor>>, // Plans if condition is false
}

/// Implementation of methods for the If struct.
impl If {
    /// Creates a new If plan.
    /// # Arguments
    /// * `cond` - The condition question.
    /// * `iftrue` - Plans to execute if true.
    /// * `iffalse` - Plans to execute if false.
    fn new(cond: Question, iftrue: Vec<Box<dyn PlanConstructor>>, iffalse: Vec<Box<dyn PlanConstructor>>) -> Self {
        If { cond, iftrue, iffalse }
    }
}

/// Implements type checking for If against a Domain.
impl Type for If {
    fn typecheck(&self, context: &Domain) -> Result<(), String> {
        self.cond.typecheck(context)?;
        for m in &self.iftrue {
            m.typecheck(context)?;
        }
        for m in &self.iffalse {
            m.typecheck(context)?;
        }
        Ok(())
    }
}

/// Formats the If for display.
impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iftrue_str: Vec<String> = self.iftrue.iter().map(|m| m.to_string()).collect();
        let iffalse_str: Vec<String> = self.iffalse.iter().map(|m| m.to_string()).collect();
        write!(f, "If('{}', {}, {})", self.cond, iftrue_str.join(", "), iffalse_str.join(", "))
    }
}

/// Trait for plan constructors.
trait PlanConstructor: Type {}

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
    latest_moves: TSet<Box<dyn Type>>, // Latest dialogue moves
    next_moves: Stack<Box<dyn Type>>, // Next moves to perform
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
    fn generate(&self, moves: &TSet<Box<dyn Type>>) -> String;

    /// Interprets an input string into a set of moves.
    /// # Arguments
    /// * `input` - The input string to interpret.
    fn interpret(&self, input: &str) -> Option<TSet<Box<dyn Type>>>;
}

/// A simple grammar for generating and interpreting dialogue moves.
struct SimpleGenGrammar {
    forms: HashMap<String, String>, // Mapping of move strings to output strings
}

/// Implementation of methods for the SimpleGenGrammar struct.
impl SimpleGenGrammar {
    /// Creates a new SimpleGenGrammar with default forms.
    fn new() -> Self {
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
    fn add_form(&mut self, move_str: &str, output: &str) {
        self.forms.insert(move_str.to_string(), output.to_string());
    }

    /// Generates a string for a single move.
    /// # Arguments
    /// * `move` - The move to generate.
    fn generate_move(&self, r#move: &dyn Type) -> String {
        if let Some(icm) = r#move.downcast_ref::<ICM>() {
            if icm.level == "per" && icm.polarity == "pos" {
                if let Some(content) = &icm.icm_content {
                    return format!("I heard you say {}", content);
                }
            }
        }
        self.forms.get(&r#move.to_string()).cloned().unwrap_or_else(|| r#move.to_string())
    }

}

/// Implements the Grammar trait for SimpleGenGrammar.
impl Grammar for SimpleGenGrammar {
    fn generate(&self, moves: &TSet<Box<dyn Type>>) -> String {
        let phrases: Vec<String> = moves.elements.iter().map(|m| self.generate_move(m.as_ref())).collect();
        self.join_phrases(&phrases)
    }

    fn interpret(&self, input: &str) -> Option<TSet<Box<dyn Type>>> {
        let result = (|| {
            if let Ok(move_val) = input.parse::<MoveEval>() {
                return Some(move_val.0);
            }
            if let Ok(ask) = Ask::new(Question::new(input)) {
                return Some(TSet {
                    elements: HashSet::from([Box::new(ask) as Box<dyn Type>]),
                    type_constraint: None,
                });
            }
            if let Ok(answer) = Answer::new(Ans::new(input)?) {
                return Some(TSet {
                    elements: HashSet::from([Box::new(answer) as Box<dyn Type>]),
                    type_constraint: None,
                });
            }
            None
        })();
        result.unwrap_or_else(|| TSet::new())
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

/// Temporary struct for parsing moves (placeholder for actual implementation).
struct MoveEval(TSet<Box<dyn Type>>);

/// Implementation of parsing for MoveEval (placeholder).
impl std::str::FromStr for MoveEval {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(MoveEval(TSet::new())) // Placeholder
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
struct TravelDB {
    entries: Vec<HashMap<String, String>>, // Database entries
}

/// Implementation of methods for the TravelDB struct.
impl TravelDB {
    /// Creates a new empty TravelDB.
    fn new() -> Self {
        TravelDB { entries: Vec::new() }
    }

    /// Adds an entry to the database.
    /// # Arguments
    /// * `entry` - The key-value map to add.
    fn add_entry(&mut self, entry: HashMap<String, String>) {
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
struct Domain {
    preds0: HashSet<String>, // Zero-place predicates
    preds1: HashMap<String, String>, // One-place predicates with their sorts
    sorts: HashMap<String, HashSet<String>>, // Sorts and their individuals
    inds: HashMap<String, String>, // Individuals and their sorts
    plans: HashMap<Question, Vec<Box<dyn PlanConstructor>>>, // Question-triggered plans
}

/// Implementation of methods for the Domain struct.
impl Domain {
    /// Creates a new Domain.
    /// # Arguments
    /// * `preds0` - Zero-place predicates.
    /// * `preds1` - One-place predicates with their sorts.
    /// * `sorts` - Sorts and their individuals.
    fn new(
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
    fn add_plan(&mut self, trigger: Question, plan: Vec<Box<dyn PlanConstructor>>) {
        for m in &plan {
            m.typecheck(self).unwrap();
        }
        trigger.typecheck(self).unwrap();
        self.plans.insert(trigger, plan);
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
    fn get_plan(&self, question: &Question) -> Option<Stack<Box<dyn PlanConstructor>>> {
        self.plans.get(question).map(|plan| {
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
        let mut private_fields = HashMap::new();
        private_fields.insert("agenda".to_string(), Box::new(Stack::new()) as Box<dyn std::any::Any>);
        private_fields.insert("plan".to_string(), Box::new(Stack::new()) as Box<dyn std::any::Any>);
        private_fields.insert("bel".to_string(), Box::new(TSet::new()) as Box<dyn std::any::Any>);
        let mut shared_fields = HashMap::new();
        shared_fields.insert("com".to_string(), Box::new(TSet::new()) as Box<dyn std::any::Any>);
        shared_fields.insert("qud".to_string(), Box::new(StackSet::new()) as Box<dyn std::any::Any>);
        let mut lu_fields = HashMap::new();
        lu_fields.insert("speaker".to_string(), Box::new(Speaker::USR) as Box<dyn std::any::Any>);
        lu_fields.insert("moves".to_string(), Box::new(TSet::new()) as Box<dyn std::any::Any>);
        let lu = Record::new(lu_fields);
        shared_fields.insert("lu".to_string(), Box::new(lu) as Box<dyn std::any::Any>);
        let shared = Record::new(shared_fields);
        let mut fields = HashMap::new();
        fields.insert("private".to_string(), Box::new(private_fields) as Box<dyn std::any::Any>);
        fields.insert("shared".to_string(), Box::new(shared) as Box<dyn std::any::Any>);
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
struct IBISController {
    is: IBISInfostate, // Information state
    mivs: StandardMIVS, // Minimal information state
    domain: Domain, // Domain knowledge
    database: TravelDB, // Travel database
    grammar: SimpleGenGrammar, // Grammar for generation and interpretation
}

/// Implementation of methods for the IBISController struct.
impl IBISController {
    /// Creates a new IBISController.
    /// # Arguments
    /// * `domain` - The domain knowledge.
    /// * `database` - The travel database.
    /// * `grammar` - The grammar for dialogue.
    fn new(domain: Domain, database: TravelDB, grammar: SimpleGenGrammar) -> Self {
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
        }
    }

    /// Selects the next moves (placeholder).
    fn select(&mut self) {
        // Placeholder: Implement selection logic
    }

    /// Generates output from the next moves.
    fn generate(&self) {
        let output = self.grammar.generate(&self.mivs.next_moves);
        self.mivs.output.set(output).unwrap();
    }

    /// Outputs the generated response.
    fn output(&self) {
        println!("S> {}", self.mivs.output.get().unwrap_or(&"[---]".to_string()));
        println!();
        self.mivs.latest_speaker.set(Speaker::SYS).unwrap();
        self.mivs.latest_moves.clear();
        self.mivs.latest_moves = self.mivs.next_moves.clone();
        self.mivs.next_moves.clear();
    }

    /// Reads user input.
    fn input(&mut self) {
        print!("U> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                input = input.trim().to_string();
                self.mivs.input.set(input).unwrap();
                self.mivs.latest_speaker.set(Speaker::USR).unwrap();
            }
            Err(_) => {
                println!("EOF");
                std::process::exit(0);
            }
        }
    }

    /// Interprets the user input into moves.
    fn interpret(&mut self) {
        self.mivs.latest_moves.clear();
        if let Some(input) = self.mivs.input.get() {
            if !input.is_empty() {
                if let Some(moves) = self.grammar.interpret(input) {
                    self.mivs.latest_moves = moves;
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
        let greet = Box::new(Greet) as Box<dyn Type>;
        self.mivs.next_moves.push(greet).unwrap();
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

// Main function to demonstrate the travel dialogue system

/// Entry point for the travel dialogue system.
fn main() {
    // Initialize zero-place predicates
    let preds0 = HashSet::from(["return".to_string(), "need-visa".to_string()]);
    
    // Initialize one-place predicates with their sorts
    let preds1 = HashMap::from([
        ("price".to_string(), "int".to_string()),
        ("how".to_string(), "means".to_string()),
        ("dest_city".to_string(), "city".to_string()),
        ("depart_city".to_string(), "city".to_string()),
        ("depart_day".to_string(), "day".to_string()),
        ("class".to_string(), "flight_class".to_string()),
        ("return_day".to_string(), "day".to_string()),
    ]);
    
    // Initialize sorts and their individuals
    let sorts = HashMap::from([
        (
            "means".to_string(),
            HashSet::from(["plane".to_string(), "train".to_string()]),
        ),
        (
            "city".to_string(),
            HashSet::from(["paris".to_string(), "london".to_string(), "berlin".to_string()]),
        ),
        (
            "day".to_string(),
            HashSet::from(["today".to_string(), "tomorrow".to_string()]),
        ),
        (
            "flight_class".to_string(),
            HashSet::from(["first".to_string(), "second".to_string()]),
        ),
    ]);
    
    // Create the domain
    let mut domain = Domain::new(preds0, preds1, sorts);

    // Define a plan for price queries
    let plan = vec![
        Box::new(Findout::new(Question::new("?x.how(x)").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(Findout::new(Question::new("?x.dest_city(x)").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(Findout::new(Question::new("?x.depart_city(x)").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(Findout::new(Question::new("?x.depart_day(x)").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(Findout::new(Question::new("?x.class(x)").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(Findout::new(Question::new("?return()").unwrap())) as Box<dyn PlanConstructor>,
        Box::new(If::new(
            Question::new("?return()").unwrap(),
            vec![Box::new(Findout::new(Question::new("?x.return_day(x)").unwrap())) as Box<dyn PlanConstructor>],
            vec![],
        )) as Box<dyn PlanConstructor>,
        Box::new(ConsultDB::new(Question::new("?x.price(x)").unwrap())) as Box<dyn PlanConstructor>,
    ];
    domain.add_plan(Question::new("?x.price(x)").unwrap(), plan);

    // Initialize the travel database
    let mut database = TravelDB::new();
    database.add_entry(HashMap::from([
        ("price".to_string(), "232".to_string()),
        ("from".to_string(), "berlin".to_string()),
        ("to".to_string(), "paris".to_string()),
        ("day".to_string(), "today".to_string()),
    ]));
    database.add_entry(HashMap::from([
        ("price".to_string(), "345".to_string()),
        ("from".to_string(), "paris".to_string()),
        ("to".to_string(), "london".to_string()),
        ("day".to_string(), "today".to_string()),
    ]));

    // Initialize the grammar
    let mut grammar = SimpleGenGrammar::new();
    grammar.add_form("Ask('?x.how(x)')", "How do you want to travel?");
    grammar.add_form("Ask('?x.dest_city(x)')", "Where do you want to go?");
    grammar.add_form("Ask('?x.depart_city(x)')", "From where are you leaving?");
    grammar.add_form("Ask('?x.depart_day(x)')", "When do you want to leave?");
    grammar.add_form("Ask('?x.return_day(x)')", "When do you want to return?");
    grammar.add_form("Ask('?x.class(x)')", "First or second class?");
    grammar.add_form("Ask('?return()')", "Do you want a return ticket?");

    // Create and run the IBIS controller
    let mut ibis = IBISController::new(domain, database, grammar);
    println!("Starting IBIS Travel Dialogue System...");
    println!("Type 'quit' to exit, or ask questions about travel.");
    println!("Example: 'I want to go to paris'");
    println!();
    ibis.run();
}