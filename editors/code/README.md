# Crabviz

A static code analysis tool that generates interactive call graph.

## Features

* Show types, methods, functions and interfaces, grouped by file
* Display function calling relationships and interface implementation relationships
* Analyze selected folders and files, or a selected function
* Export call graphs to svg files

## Requirements

Crabviz utilizes the "call hierarchy" capability of LSP server under the hood, so if you want to analyze your project with Crabviz, you should have a corresponding language server extension with "call hierarchy" support.

## Usage

* Generate a call graph for selected files and folders

    ![call graph for files](https://user-images.githubusercontent.com/20551552/242812058-60584f59-a8f0-4a56-90eb-373c3f3b8cd5.gif)

    Select the files and folders (support multiple selections) you want to analyze, right click and select `Crabviz: Generate Call Graph` in the context menu.

    You will then see "Crabviz: Generating call graph" in the status bar. Once the analysis is complete, the result will be displayed on a new page.

* Generate a call graph for the selected function

    ![call graph for the selected function](https://user-images.githubusercontent.com/20551552/287503577-1f0a4155-313c-44cd-a4e3-a8b0ce786eca.gif)

    Right click on the function you want to analyze, and select `Crabviz: Generate Function Call Graph` in the context menu.
