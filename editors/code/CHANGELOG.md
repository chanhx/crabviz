# Change Log

## 0.3.2 (2024.1.23)

### Fixed

* Error in language detection ([#25](https://github.com/chanhx/crabviz/issues/25))

## 0.4.0 (2024.3.27)

### Improvements

* New color scheme
* Add icons on cells to distinguish between enum, struct and class
* Show progress when generating call graph for selected files
* Allow cancellations when generating call graphs for selected files
* Check git ignore rules in function call graph generation
* Make interface cells clickable

### Fixed

* Function call graph generation error when involving recursive functions
* Call graph generation errors on Windows

## 0.3.1 (2023.12.24)

### Fixed

* Module loading issue

## 0.3.0 (2023.12.4)

### Features

* Generate a call graph for a selected function
* Filter out test files when analyzing Go projects

### Fixed

* Calls from nested functions are not shown for Go projects

## 0.2.0 (2023.9.24)

### Features

* Export call graph

## 0.1.2 (2023.9.7)

### Fixed

* Edges are not rendered for some languages or certain calls

## 0.1.1 (2023.6.17)

### Fixed

* Focus won't get cleared when dragging the graph
