import * as vscode from 'vscode';
import * as lsp_types from 'vscode-languageserver-types';

export function convertSymbol(symbol: vscode.DocumentSymbol): lsp_types.DocumentSymbol {
	return lsp_types.DocumentSymbol.create(
		symbol.name,
		symbol.detail,
		convertSymbolKind(symbol.kind),
		symbol.range,
		symbol.selectionRange,
		symbol.children.map(convertSymbol),
	);
}

export function convertSymbolKind(kind: vscode.SymbolKind): lsp_types.SymbolKind {
	switch (kind) {
		case vscode.SymbolKind.File:
			return lsp_types.SymbolKind.File;
		case vscode.SymbolKind.Module:
			return lsp_types.SymbolKind.Module;
		case vscode.SymbolKind.Namespace:
			return lsp_types.SymbolKind.Namespace;
		case vscode.SymbolKind.Package:
			return lsp_types.SymbolKind.Package;
		case vscode.SymbolKind.Class:
			return lsp_types.SymbolKind.Class;
		case vscode.SymbolKind.Method:
			return lsp_types.SymbolKind.Method;
		case vscode.SymbolKind.Property:
			return lsp_types.SymbolKind.Property;
		case vscode.SymbolKind.Field:
			return lsp_types.SymbolKind.Field;
		case vscode.SymbolKind.Constructor:
			return lsp_types.SymbolKind.Constructor;
		case vscode.SymbolKind.Enum:
			return lsp_types.SymbolKind.Enum;
		case vscode.SymbolKind.Interface:
			return lsp_types.SymbolKind.Interface;
		case vscode.SymbolKind.Function:
			return lsp_types.SymbolKind.Function;
		case vscode.SymbolKind.Variable:
			return lsp_types.SymbolKind.Variable;
		case vscode.SymbolKind.Constant:
			return lsp_types.SymbolKind.Constant;
		case vscode.SymbolKind.String:
			return lsp_types.SymbolKind.String;
		case vscode.SymbolKind.Number:
			return lsp_types.SymbolKind.Number;
		case vscode.SymbolKind.Boolean:
			return lsp_types.SymbolKind.Boolean;
		case vscode.SymbolKind.Array:
			return lsp_types.SymbolKind.Array;
		case vscode.SymbolKind.Object:
			return lsp_types.SymbolKind.Object;
		case vscode.SymbolKind.Key:
			return lsp_types.SymbolKind.Key;
		case vscode.SymbolKind.Null:
			return lsp_types.SymbolKind.Null;
		case vscode.SymbolKind.EnumMember:
			return lsp_types.SymbolKind.EnumMember;
		case vscode.SymbolKind.Struct:
			return lsp_types.SymbolKind.Struct;
		case vscode.SymbolKind.Event:
			return lsp_types.SymbolKind.Event;
		case vscode.SymbolKind.Operator:
			return lsp_types.SymbolKind.Operator;
		case vscode.SymbolKind.TypeParameter:
			return lsp_types.SymbolKind.TypeParameter;
	}
}
