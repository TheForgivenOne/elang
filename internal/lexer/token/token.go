// Package token defines the token kinds of the elang lexer.
//
// elang source is a dense symbol stream: whitespace is insignificant except
// that a newline terminates a statement. Sigils never appear inside
// identifiers, which is what makes zero-whitespace packing unambiguous.
package token

// Kind identifies the lexical class of a token.
type Kind int

const (
	EOF Kind = iota
	ILLEGAL
	NEWLINE

	IDENT     // [a-z_][a-z0-9_]*  bindings and functions
	TYPEIDENT // [A-Z_][A-Za-z0-9_]*  type names
	BOOL      // t | f
	INT       // 123, 1_000
	FLOAT     // 1.5
	STRING    // "..."

	// Sigils. Single characters are their own kind; multi-character
	// operators are distinct kinds so the parser never re-splits them.
	ASSIGN    // =
	EQ        // ==
	NEQ       // !=
	LT        // <
	LE        // <=
	GT        // >
	GE        // >=
	BANG      // !
	QUESTION  // ?
	TILDE     // ~
	AT        // @
	HASH      // #
	CARET     // ^
	PIPE      // |
	PIPLINE   // |>
	FATARROW  // =>
	COLON     // :
	COMMA     // ,
	DOT       // .
	SEMICOLON // ;
	AMP       // &
	PLUS      // +
	MINUS     // -
	STAR      // *
	SLASH     // /
	CONTINUE  // \  explicit line continuation

	LPAREN // (
	RPAREN // )
	LBRACE // {
	RBRACE // }
	LBRACK // [
	RBRACK // ]
)

// descriptions maps every kind to its printed form, used by Token.String
// and by tests. Kinds are numbered densely from EOF, so a mismatch between
// the enum and this table is a compile-time-free but test-time-obvious bug;
// TestKindDescriptionCoversAll guards it.
var descriptions = map[Kind]string{
	EOF:       "EOF",
	ILLEGAL:   "ILLEGAL",
	NEWLINE:   "NEWLINE",
	IDENT:     "IDENT",
	TYPEIDENT: "TYPEIDENT",
	BOOL:      "BOOL",
	INT:       "INT",
	FLOAT:     "FLOAT",
	STRING:    "STRING",
	ASSIGN:    "=",
	EQ:        "==",
	NEQ:       "!=",
	LT:        "<",
	LE:        "<=",
	GT:        ">",
	GE:        ">=",
	BANG:      "!",
	QUESTION:  "?",
	TILDE:     "~",
	AT:        "@",
	HASH:      "#",
	CARET:     "^",
	PIPE:      "|",
	PIPLINE:   "|>",
	FATARROW:  "=>",
	COLON:     ":",
	COMMA:     ",",
	DOT:       ".",
	SEMICOLON: ";",
	AMP:       "&",
	PLUS:      "+",
	MINUS:     "-",
	STAR:      "*",
	SLASH:     "/",
	CONTINUE:  "\\",
	LPAREN:    "(",
	RPAREN:    ")",
	LBRACE:    "{",
	RBRACE:    "}",
	LBRACK:    "[",
	RBRACK:    "]",
}

// Description returns the printed form of the kind.
func (k Kind) Description() string {
	if d, ok := descriptions[k]; ok {
		return d
	}
	return "UNKNOWN"
}

// Token is a single lexed token with its source position. Line and Col are
// 1-based; Col is counted in bytes, not runes, so error positions are byte
// exact under packed input.
type Token struct {
	Kind Kind
	Lit  string
	Line int
	Col  int
}

// String renders the token for diagnostics and test failures.
func (t Token) String() string {
	switch t.Kind {
	case IDENT, TYPEIDENT, INT, FLOAT, STRING, BOOL:
		return t.Kind.Description() + "(" + t.Lit + ")"
	case NEWLINE, EOF, ILLEGAL:
		return t.Kind.Description()
	default:
		return t.Kind.Description()
	}
}
