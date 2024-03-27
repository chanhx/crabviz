# Crabviz

Crabviz is a [LSP](https://microsoft.github.io/language-server-protocol/)-based call graph generator. It leverages the Language Server Protocol to generate interactive call graphs, helps you visually explore source code.

## Features

* Workable for various programming languages
* Highlight on click
* Two kinds of graphs

   You can generate a call graph for selected files to get an overview, or a selected function to track the call hierarchy.
* Export call graphs as SVG

## Preview

![preview](https://raw.githubusercontent.com/chanhx/assets/main/crabviz/preview.gif)

## Install

Since Crabviz utilizes the capabilities of language servers, it is better suited as an IDE/editor extension than a standalone command line tool.

It is currently available on [VS Code](https://marketplace.visualstudio.com/items?itemName=chanhx.crabviz), and PRs for other editors are welcome.

## Credits

Crabviz is inspired by [graphql-voyager](https://github.com/graphql-kit/graphql-voyager) and [go-callvis](https://github.com/ondrajz/go-callvis).
