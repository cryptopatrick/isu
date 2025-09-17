# isu

**A Rust implementation of Information State Update (ISU) theory for dialogue management and conversational AI systems**

[![Crates.io](https://img.shields.io/crates/v/isu.svg)](https://crates.io/crates/isu)
[![Documentation](https://docs.rs/isu/badge.svg)](https://docs.rs/isu)
[![License](https://img.shields.io/badge/license-MIT%2FUnlicense-blue.svg)](https://github.com/cryptopatrick/isu)

## Overview

`isu` is a comprehensive Rust library that implements Information State Update (ISU) theory, a formal framework for dialogue management in conversational AI systems. This library provides the core components needed to build sophisticated dialogue managers following the Issue-Based Information State (IBIS) approach.

## Key Features

### 📊 **Core Data Structures**
- **Value**: Generic containers with type constraints and validation
- **Stack/StackSet**: LIFO data structures with uniqueness guarantees
- **TSet**: Typed sets with optional type checking
- **Record**: Key-value stores with dynamic type validation

### 🗣️ **Dialogue Management**
- **IBIS Controller**: Full implementation of Issue-Based Information State dialogue management
- **Information State**: Tracks dialogue context, beliefs, commitments, and questions under discussion (QUD)
- **Dialogue Moves**: Support for greetings, questions, answers, and ICM (Information State Update Control Mechanisms)

### 🧠 **Semantic Types & Questions**
- **Propositions**: Structured representation of facts with polarity
- **Questions**: Support for wh-questions, yes/no questions, and alternative questions
- **Individuals & Predicates**: Typed semantic objects with domain validation
- **Answer Types**: Short answers, propositions, and yes/no responses

### 📝 **Natural Language Processing**
- **Grammar System**: Configurable generation and interpretation of dialogue moves
- **CFG Support**: Context-free grammar parsing capabilities
- **Multi-modal Input**: Support for different input handling strategies (interactive, demo, batch)

### 🗄️ **Knowledge Management**
- **Domain Knowledge**: Formal representation of predicates, sorts, and individuals
- **Database Integration**: Query interface for external knowledge sources
- **Plan Constructors**: Conditional planning with findout, consult, and response strategies

### 🎯 **Travel Domain Example**
- Complete implementation of a travel booking dialogue system
- Demonstrates price queries, destination planning, and multi-turn conversations
- Includes sample database entries and conversational flows

## Architecture

The library follows ISU theory principles:

1. **Information State**: Central repository for dialogue context
2. **Update Rules**: Formal rules for state transitions
3. **Dialogue Management**: Control flow for turn-taking and planning
4. **Semantic Interpretation**: Mapping between natural language and formal representations

## Quick Start

```rust
use isu::*;
use std::collections::{HashMap, HashSet};

// Create domain knowledge
let preds0 = HashSet::from(["expensive".to_string()]);
let preds1 = HashMap::from([("city".to_string(), "location".to_string())]);
let sorts = HashMap::from([
    ("location".to_string(), HashSet::from(["paris".to_string(), "london".to_string()]))
]);

let domain = Domain::new(preds0, preds1, sorts);
let database = TravelDB::new();
let grammar = SimpleGenGrammar::new();

// Create dialogue manager
let mut ibis = IBISController::new(domain, database, grammar);

// Run interactive dialogue
ibis.run();
```

## Use Cases

- **Chatbots & Virtual Assistants**: Build sophisticated conversational agents
- **Dialogue Research**: Experiment with ISU theory and dialogue management strategies  
- **Task-Oriented Systems**: Implement goal-driven dialogue systems (booking, support, etc.)
- **Multi-turn Conversations**: Handle complex dialogue flows with context tracking
- **Educational Tools**: Learn and teach dialogue management concepts

## Documentation

Comprehensive documentation is available at [docs.rs/isu](https://docs.rs/isu), including:
- API reference for all public types and functions
- Tutorial on building dialogue systems
- Examples of different dialogue management strategies
- Performance considerations and best practices

## Examples

The `examples/` directory contains:
- `travel.rs`: Complete travel booking dialogue system
- Domain-specific implementations and configurations
- Various input handling strategies (interactive, demo, batch)

## Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for details on:
- Code style and testing requirements
- Submitting bug reports and feature requests
- Development setup and workflow

## License

This project is dual-licensed under MIT and Unlicense. See [LICENSE-MIT](LICENSE-MIT) and [UNLICENSE](UNLICENSE) for details.