// Package lexer implements elang's lexer.
//
// The lexer is sigil-first and whitespace-insignificant: it must tokenize
// input that the AI has packed with zero spaces, such as
//
//	o=P.new(0,0);!v=V.new();v.push(Shape.C(3))
//
// The one positional rule it enforces is that a newline terminates a
// statement, so a run of newlines collapses to a single NEWLINE token and
// the parser decides where NEWLINE is meaningful.
package lexer

import (
	"fmt"

	"github.com/TheForgivenOne/elang/internal/lexer/token"
)

// Error is a lexical error with its exact byte position.
type Error struct {
	Line int
	Col  int
	Msg  string
}

func (e *Error) Error() string {
	return fmt.Sprintf("elang: lex error at %d:%d: %s", e.Line, e.Col, e.Msg)
}

// Lexer scans elang source into tokens.
type Lexer struct {
	src  []byte
	pos  int
	line int
	col  int

	toks []token.Token
	errs []*Error
}

// New returns a lexer over src. Positions are 1-based and counted in bytes,
// so diagnostics point at exact byte offsets under packed input.
func New(src []byte) *Lexer {
	return &Lexer{src: src, line: 1, col: 1}
}

// Tokenize returns every token in src (terminated by NEWLINE and EOF) and
// every lexical error found. Errors do not stop tokenization: the AI gets
// the full diagnostic list in one pass.
func Tokenize(src []byte) ([]token.Token, []*Error) {
	l := New(src)
	l.run()
	return l.toks, l.errs
}

func (l *Lexer) run() {
	l.skipBOM()
	for l.pos < len(l.src) {
		c := l.src[l.pos]
		switch {
		case c == '\n':
			l.emitNewline()
			l.advance()
		case c == '\r':
			// Bare CR carries no meaning; LF owns statement termination.
			l.advance()
		case c == ' ' || c == '\t' || c == '\v' || c == '\f':
			l.advance()
		case c == '%':
			l.skipLineComment()
		case c == '"':
			l.lexString()
		case isDigit(c):
			l.lexNumber()
		case isIdentStart(c) || isUpper(c):
			// Lowercase starts a binding identifier, uppercase a type
			// identifier; both are scanned by the same routine.
			l.lexIdent()
		default:
			if !l.lexSymbol() {
				return
			}
		}
	}
	// Collapse away if the source already ended on a newline.
	l.emitNewline()
	l.emitAt(token.EOF, "", l.line, l.col)
}

// advance consumes one byte, tracking the byte column for exact positions.
func (l *Lexer) advance() byte {
	c := l.src[l.pos]
	l.pos++
	if c == '\n' {
		l.line++
		l.col = 1
	} else {
		l.col++
	}
	return c
}

func (l *Lexer) peek() byte {
	if l.pos < len(l.src) {
		return l.src[l.pos]
	}
	return 0
}

func (l *Lexer) peekAt(off int) byte {
	if l.pos+off < len(l.src) {
		return l.src[l.pos+off]
	}
	return 0
}

func (l *Lexer) skipBOM() {
	if l.pos+2 < len(l.src) && l.src[l.pos] == 0xEF && l.src[l.pos+1] == 0xBB && l.src[l.pos+2] == 0xBF {
		l.pos += 3
		l.col += 3
	}
}

// emitAt records a token at an explicitly captured start position. Every
// scan function captures its position before consuming, because the escape
// forms (strings, line continuations) consume more bytes than the literal.
func (l *Lexer) emitAt(k token.Kind, lit string, line, col int) {
	l.toks = append(l.toks, token.Token{Kind: k, Lit: lit, Line: line, Col: col})
}

// emitNewline collapses runs of blank lines into one NEWLINE and never emits
// a leading NEWLINE, so the parser's skip logic stays trivial.
func (l *Lexer) emitNewline() {
	if len(l.toks) == 0 {
		return
	}
	if last := l.toks[len(l.toks)-1]; last.Kind == token.NEWLINE {
		return
	}
	l.emitAt(token.NEWLINE, "", l.line, l.col)
}

func (l *Lexer) errorAt(line, col int, format string, args ...any) {
	l.errs = append(l.errs, &Error{Line: line, Col: col, Msg: fmt.Sprintf(format, args...)})
}

func (l *Lexer) errorf(format string, args ...any) {
	l.errorAt(l.line, l.col, format, args...)
}

func (l *Lexer) skipLineComment() {
	for l.pos < len(l.src) && l.src[l.pos] != '\n' {
		l.advance()
	}
}

func (l *Lexer) lexString() {
	startLine, startCol := l.line, l.col
	l.advance() // opening quote
	var lit []byte
	for l.pos < len(l.src) {
		c := l.src[l.pos]
		switch c {
		case '"':
			l.advance()
			l.emitAt(token.STRING, string(lit), startLine, startCol)
			return
		case '\n':
			l.errorAt(startLine, startCol, "unterminated string literal")
			return
		case '\\':
			l.advance()
			if l.pos >= len(l.src) {
				l.errorAt(startLine, startCol, "unterminated string literal")
				return
			}
			switch e := l.advance(); e {
			case 'n':
				lit = append(lit, '\n')
			case 't':
				lit = append(lit, '\t')
			case '\\':
				lit = append(lit, '\\')
			case '"':
				lit = append(lit, '"')
			case '0':
				lit = append(lit, 0)
			default:
				l.errorAt(startLine, startCol, "unknown escape sequence \\%c in string literal", e)
				return
			}
		default:
			lit = append(lit, l.advance())
		}
	}
	l.errorAt(startLine, startCol, "unterminated string literal")
}

func (l *Lexer) lexNumber() {
	startLine, startCol := l.line, l.col
	var lit []byte
	for l.pos < len(l.src) && (isDigit(l.src[l.pos]) || l.src[l.pos] == '_') {
		lit = append(lit, l.advance())
	}
	isFloat := false
	if l.peek() == '.' && isDigit(l.peekAt(1)) {
		isFloat = true
		lit = append(lit, l.advance()) // .
		for l.pos < len(l.src) && (isDigit(l.src[l.pos]) || l.src[l.pos] == '_') {
			lit = append(lit, l.advance())
		}
	}
	// A numeric literal must not touch an identifier character: under
	// zero-whitespace packing, "1abc" would otherwise read two ways.
	if isIdentContinue(l.peek()) {
		for l.pos < len(l.src) && isIdentContinue(l.src[l.pos]) {
			lit = append(lit, l.advance())
		}
		l.errorAt(startLine, startCol, "numeric literal %q immediately followed by an identifier character", string(lit))
		return
	}
	if isFloat {
		l.emitAt(token.FLOAT, string(lit), startLine, startCol)
	} else {
		l.emitAt(token.INT, string(lit), startLine, startCol)
	}
}

func (l *Lexer) lexIdent() {
	startLine, startCol := l.line, l.col
	var lit []byte
	for l.pos < len(l.src) && isIdentContinue(l.src[l.pos]) {
		lit = append(lit, l.advance())
	}
	s := string(lit)
	switch {
	case s == "t" || s == "f":
		// Boolean constants. t and f are reserved: they are not identifiers,
		// which keeps them unambiguous in packed expression position.
		l.emitAt(token.BOOL, s, startLine, startCol)
	case isUpper(lit[0]):
		l.emitAt(token.TYPEIDENT, s, startLine, startCol)
	default:
		l.emitAt(token.IDENT, s, startLine, startCol)
	}
}

// lexSymbol scans sigils and multi-character operators, returning false when
// the current byte cannot start any token.
func (l *Lexer) lexSymbol() bool {
	startLine, startCol := l.line, l.col
	two := ""
	if l.pos+1 < len(l.src) {
		two = string(l.src[l.pos : l.pos+2])
	}
	switch two {
	case "==", "!=", "<=", ">=", "|>", "=>":
		l.advance()
		l.advance()
		l.emitAt(twoCharKind(two), two, startLine, startCol)
		return true
	}
	c := l.advance()
	switch c {
	case '=':
		l.emitAt(token.ASSIGN, "=", startLine, startCol)
	case '<':
		l.emitAt(token.LT, "<", startLine, startCol)
	case '>':
		l.emitAt(token.GT, ">", startLine, startCol)
	case '!':
		l.emitAt(token.BANG, "!", startLine, startCol)
	case '?':
		l.emitAt(token.QUESTION, "?", startLine, startCol)
	case '~':
		l.emitAt(token.TILDE, "~", startLine, startCol)
	case '@':
		l.emitAt(token.AT, "@", startLine, startCol)
	case '#':
		l.emitAt(token.HASH, "#", startLine, startCol)
	case '^':
		l.emitAt(token.CARET, "^", startLine, startCol)
	case '|':
		l.emitAt(token.PIPE, "|", startLine, startCol)
	case ':':
		l.emitAt(token.COLON, ":", startLine, startCol)
	case ',':
		l.emitAt(token.COMMA, ",", startLine, startCol)
	case '.':
		l.emitAt(token.DOT, ".", startLine, startCol)
	case ';':
		l.emitAt(token.SEMICOLON, ";", startLine, startCol)
	case '&':
		l.emitAt(token.AMP, "&", startLine, startCol)
	case '+':
		l.emitAt(token.PLUS, "+", startLine, startCol)
	case '-':
		l.emitAt(token.MINUS, "-", startLine, startCol)
	case '*':
		l.emitAt(token.STAR, "*", startLine, startCol)
	case '/':
		l.emitAt(token.SLASH, "/", startLine, startCol)
	case '(':
		l.emitAt(token.LPAREN, "(", startLine, startCol)
	case ')':
		l.emitAt(token.RPAREN, ")", startLine, startCol)
	case '{':
		l.emitAt(token.LBRACE, "{", startLine, startCol)
	case '}':
		l.emitAt(token.RBRACE, "}", startLine, startCol)
	case '[':
		l.emitAt(token.LBRACK, "[", startLine, startCol)
	case ']':
		l.emitAt(token.RBRACK, "]", startLine, startCol)
	case '\\':
		if l.peek() != '\n' {
			l.errorAt(startLine, startCol, "stray '\\': only valid as a line continuation at end of line")
			return false
		}
		l.advance() // the newline it escapes
		l.emitAt(token.CONTINUE, "\\", startLine, startCol)
	default:
		l.errorAt(startLine, startCol, "unexpected character %q", string(c))
		return false
	}
	return true
}

func twoCharKind(s string) token.Kind {
	switch s {
	case "==":
		return token.EQ
	case "!=":
		return token.NEQ
	case "<=":
		return token.LE
	case ">=":
		return token.GE
	case "|>":
		return token.PIPLINE
	case "=>":
		return token.FATARROW
	}
	return token.ILLEGAL
}

func isDigit(c byte) bool { return c >= '0' && c <= '9' }

func isUpper(c byte) bool { return c >= 'A' && c <= 'Z' }

func isLower(c byte) bool { return c >= 'a' && c <= 'z' }

// isIdentStart reports whether c can start an identifier (lowercase or _).
// Uppercase starts a type identifier instead.
func isIdentStart(c byte) bool { return isLower(c) || c == '_' }

func isIdentContinue(c byte) bool {
	return isLower(c) || isUpper(c) || isDigit(c) || c == '_'
}
