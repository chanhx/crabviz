
export function groupFileExtensions(extensions: Set<string>): { [lang: string]: string[] } {
  const extensionsByLanguage: {[lang: string]: string[]} = {};

		for (const ext of extensions) {
			const lang = languagesByExtension[ext];
			if (!lang) {
				continue;
			}

			if (lang in extensionsByLanguage) {
				extensionsByLanguage[lang].push(ext);
			} else {
				extensionsByLanguage[lang] = [ext];
			}
		}

    return extensionsByLanguage;
}

const languagesByExtension: { [ext: string] : string } = {
  'c': 'C/C++',
  'cc': 'C/C++',
  'cpp': 'C/C++',
  'c++': 'C/C++',
  'cp': 'C/C++',
  'cxx': 'C/C++',
  'h': 'C/C++',
  'h++': 'C/C++',
  'hh': 'C/C++',
  'hpp': 'C/C++',
  'hxx': 'C/C++',

  'ada': 'Ada',
  'ads': 'Ada',
  'adb': 'Ada',
  'as': 'ActionScript',

  'ceylon': 'Ceylon',
  'cl': 'Common Lisp',
  'clj': 'Clojure',
  'cljc': 'Clojure',
  'cljs': 'Clojure',
  'cr': 'Crystal',
  'cs': 'C#',
  'coq': 'Coq',
  'coffee': 'CoffeeScript',
  'cob': 'Cobol',
  'cbl': 'Cobol',
  'ccp': 'Cobol',
  'cobol': 'Cobol',
  'cpy': 'Cobol',

  'd': 'D',
  'dart': 'Dart',

  'e': 'Eiffel',
  'el': 'Emacs Lisp',
  'elm': 'Elm',
  'erl': 'Erlang',
  'ex': 'Elixir',
  'exs': 'Elixir',

  'fs': 'F#',
  'fsi': 'F#',
  'fsx': 'F#',

  'f90': 'Fortran',
  'f': 'Fortran',
  'f03': 'Fortran',
  'f08': 'Fortran',
  'f77': 'Fortran',
  'f95': 'Fortran',
  'for': 'Fortran',
  'fpp': 'Fortran',

  'go': 'Go',
  'groovy': 'Groovy',
  'gvy': 'Groovy',
  'gy': 'Groovy',
  'gsh': 'Groovy',

  'hrl': 'Erlang',
  'hs': 'Haskell',
  'lhs': 'Haskell',

  'idr': 'Idris',
  'lidr': 'Idris',
  'ijs': 'J',

  'java': 'Java',
  'jl': 'Julia',
  'js': 'Javascript',

  'kt': 'Kotlin',
  'kts': 'Kotlin',

  'lua': 'Lua',
  'lisp': 'Common Lisp',
  'lsp': 'Common Lisp',

  'm': '*.m',
  'ml': 'OCaml',
  'mli': 'OCaml',
  'nim': 'Nim',
  'nix': 'Nix',

  'php': 'PHP',
  'pl': 'Perl',
  // 'pl': 'Prolog',
  'pm': 'Perl',
  'pony': 'Pony',
  'purs': 'PureScript',
  'py': 'Python',

  'pas': 'Pacal',
  'dfm': 'Pacal',
  'dpr': 'Pacal',
  'inc': 'Pacal',
  'lpr': 'Pacal',
  'pp': 'Pacal',

  'r': 'R',
  'rd': 'R',
  'rsx': 'R',
  'rb': 'Ruby',
  'rkt': 'Racket',
  'rs': 'Rust',

  'raku': 'Raku',
  'p6':   'Raku',
  'pl6':  'Raku',
  'pm6':  'Raku',
  'pod6': 'Raku',

  'scala': 'Scala',
  'scm': 'Scheme',
  'ss': 'Scheme',
  'sml': 'Standard ML',
  'st': 'Smalltalk',
  'swift': 'Swift',

  'tcl': 'Tcl',
  'ts': 'Typescript',

  'vb': 'Visual Basic',

  'zig': 'Zig',
};
