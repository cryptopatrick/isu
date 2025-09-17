use isu::*;
use std::collections::{HashMap, HashSet};

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
        "Findout('?x.how(x)')".to_string(),
        "Findout('?x.dest_city(x)')".to_string(),
        "Findout('?x.depart_city(x)')".to_string(),
        "Findout('?x.depart_day(x)')".to_string(),
        "Findout('?x.class(x)')".to_string(),
        "Findout('?return()')".to_string(),
        "If('?return()', ['Findout(?x.return_day(x))'], [])".to_string(),
        "ConsultDB('?x.price(x)')".to_string(),
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

    // Create demo inputs for non-interactive testing
    let demo_inputs = vec![
        "I want to go to paris".to_string(),
        "train".to_string(),
        "berlin".to_string(),
        "today".to_string(),
        "first".to_string(),
        "yes".to_string(),
        "tomorrow".to_string(),
        "quit".to_string(),
    ];
    
    // Create the IBIS controller with demo input handler
    let demo_handler = isu::DemoInputHandler::new(demo_inputs);
    let mut ibis = isu::IBISController::with_input_handler(domain, database, grammar, Box::new(demo_handler));
    
    println!("Starting IBIS Travel Dialogue System (Demo Mode)...");
    println!("Simulating user interaction with predefined inputs:");
    println!();
    
    // Run the demo
    ibis.run();
}