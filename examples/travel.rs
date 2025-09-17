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