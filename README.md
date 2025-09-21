<h1 align="center">
  <br>
    <img 
      src="https://github.com/cryptopatrick/factory/blob/master/img/100days/isu.png" 
      alt="Title" 
      width="200"
    />
  <br>
  ISU
  <br>
</h1>

<h4 align="center">
  Rust implementation of 
  <a href="https://link.springer.com/chapter/10.1007/978-94-010-0019-2_15" target="_blank">
    Information State Update</a>.</h4>

<p align="center">
  <a href="https://crates.io/crates/isu" target="_blank">
    <img src="https://img.shields.io/crates/v/isu" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/isu" target="_blank">
    <img src="https://img.shields.io/crates/d/isu" alt="Downloads"/>
  </a>
  <a href="https://github.com/sulu/sulu/actions" target="_blank">
    <img src="https://img.shields.io/github/actions/workflow/status/sulu/sulu/test-application.yaml" alt="Test workflow status"/>
  </a>
  <a href="https://docs.rs/isu" target="_blank">
    <img src="https://docs.rs/isu/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/github/license/sulu/sulu.svg" alt="GitHub license"/>
  </a>
</p>

<p align="center">
  <a href="#-what-is-isu">What is ISU</a> •
  <a href="#-features">Features</a> •
  <a href="#-how-to-use">How To Use</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

<!--
![screenshot](https://github.com/cryptopatrick/factory/blob/master/img/markdownify.gif)
-->


## 🛎 Important Notices
* `master` branch file paths are **not** considered stable. [Verify your repository URI references](#unstable-file-paths)
* cloning this repository is **not** recommended ([due to Repo size](#option-9-clone-the-repo)) unless you are going to be [contributing to development](#contributing)


<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#-what-is-isu"> What is ISU</a></li>
    <li><a href="#-features"> Features</a></li>
      <ul>
        <li><a href="#-core-data-structures"> Core Data Structures</a></li>
        <li><a href="#-dialogue-management">Dialogue Management</a></li>
        <li><a href="#-semantic-types-&-questions">Semantic Types & Questions</a></li>
        <li><a href="#-natural-language-processing">Natural Language Processing</a></li>
      </ul>
    <li><a href="#-how-to-use"> How to Use</a></li>
    <li><a href="#-documentation"> Documentation</a></li>
    <li><a href="#-author"> Author</a></li>
    <li><a href="#-support"> Support</a>
    <li><a href="#-contributing"> Contributing</a></li>
    <li><a href="#-license">License</a></li>
    <!--
    <li><a href="#experiments">Experiments</a></li>
    <li><a href="#results-and-discussion"> Results and Discussion</a></li>
    <li><a href="#references"> References</a></li>
    <li><a href="#contributors"> Contributors</a></li>
    -->
    </li>
  </ol>
</details>





## 🤔 What is ISU

`isu` is a comprehensive Rust library that implements Information State Update (ISU) theory, a formal framework for dialogue management in conversational AI systems. This library provides the core components needed to build sophisticated dialogue managers following the Issue-Based Information State (IBIS) approach.

### Use Cases

- **Chatbots & Virtual Assistants**: Build sophisticated conversational agents
- **Dialogue Research**: Experiment with ISU theory and dialogue management strategies  
- **Task-Oriented Systems**: Implement goal-driven dialogue systems (booking, support, etc.)
- **Multi-turn Conversations**: Handle complex dialogue flows with context tracking
- **Educational Tools**: Learn and teach dialogue management concepts

### Architecture

The library follows ISU theory principles:

1. **Information State**: Central repository for dialogue context
2. **Update Rules**: Formal rules for state transitions
3. **Dialogue Management**: Control flow for turn-taking and planning
4. **Semantic Interpretation**: Mapping between natural language and formal representations

## 📷 Features

###  Core Data Structures
- **Value**: Generic containers with type constraints and validation
- **Stack/StackSet**: LIFO data structures with uniqueness guarantees
- **TSet**: Typed sets with optional type checking
- **Record**: Key-value stores with dynamic type validation

###  **Dialogue Management**
- **IBIS Controller**: Full implementation of Issue-Based Information State dialogue management
- **Information State**: Tracks dialogue context, beliefs, commitments, and questions under discussion (QUD)
- **Dialogue Moves**: Support for greetings, questions, answers, and ICM (Information State Update Control Mechanisms)

###  **Semantic Types & Questions**
- **Propositions**: Structured representation of facts with polarity
- **Questions**: Support for wh-questions, yes/no questions, and alternative questions
- **Individuals & Predicates**: Typed semantic objects with domain validation
- **Answer Types**: Short answers, propositions, and yes/no responses

###  **Natural Language Processing**
- **Grammar System**: Configurable generation and interpretation of dialogue moves
- **CFG Support**: Context-free grammar parsing capabilities
- **Multi-modal Input**: Support for different input handling strategies (interactive, demo, batch)

###  **Knowledge Management**
- **Domain Knowledge**: Formal representation of predicates, sorts, and individuals
- **Database Integration**: Query interface for external knowledge sources
- **Plan Constructors**: Conditional planning with findout, consult, and response strategies

### **Travel Domain Example**
- Complete implementation of a travel booking dialogue system
- Demonstrates price queries, destination planning, and multi-turn conversations
- Includes sample database entries and conversational flows



## 🚙 How to Use

### Requirements
Revup requires python 3.8 or higher and git 2.43 or higher. Revup works with Linux, OSX, and Windows (limited testing).
Follow instructions here to get the latest git version for your OS. Revup uses flags only present in newer git versions.

### Installation

Install with cargo.

```bash
cargo add isu
```
### Example

The `examples/` directory contains:
- `travel.rs`: Complete travel booking dialogue system
- Domain-specific implementations and configurations
- Various input handling strategies (interactive, demo, batch)

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

## 📚 Documentation

Comprehensive documentation is available at [docs.rs/isu](https://docs.rs/isu), including:
- API reference for all public types and functions
- Tutorial on building dialogue systems
- Examples of different dialogue management strategies
- Performance considerations and best practices


## 🖊 Author

<span>CryptoPatrick  <a href="https://x.com/cryptopatrick"><img width="30" height="30" src="https://github.com/cryptopatrick/factory/blob/master/img/x.png" /></a>  </span>  
Keybase Verification:  
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support
Leave a ⭐ If you think this project is cool.  
If you think it has helped in any way, consider [!buying me a coffee!](https://github.com/cryptopatrick/factory/blob/master/img/bmc-button.png)

## 🤝 Contributing

Found a bug? Missing a specific feature?
Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md) for details on:
- Code style and testing requirements
- Submitting bug reports and feature requests
- Development setup and workflow

## 🗄 License
This project is licensed under MIT. See [LICENSE](LICENSE) for details.


