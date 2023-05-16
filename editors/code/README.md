# Crabviz

A static code analysis tool that generates interactive call graph.

## Features

* group functions/methods by file
* display function calling relationships and interface implementation relationships

## Preview

![preview](https://user-images.githubusercontent.com/20551552/238610380-e7cfc0e4-9bc1-4fc2-ad19-140d9c5a687f.gif)

## Requirements

Crabviz utilizes the capabilities of LSP server, so if you want to analyze your project with Crabviz, you should have the corresponding language server extension installed.

## Commands

**Crabviz: Generate Call Graph**

This command detects languages in the workspace, and then opens a quick pick so that you can pick a language to generate its call graph.
