package lexer

import (
	"strings"
	"testing"

	"github.com/TheForgivenOne/elang/internal/lexer/token"
)

// kinds renders a token stream as a compact kind list, with literals kept
// where they disambiguate. Used by most tests below.
func kinds(toks []token.Token) []string {
	out := make([]string, 0, len(toks))
	for _, tk := range toks {
		switch tk.Kind {
		case token.IDENT, token.TYPEIDENT, token.INT, token.FLOAT, token.STRING, token.BOOL:
			out = append(out, tk.String())
		case token.NEWLINE:
			out = append(out, "NL")
		case token.EOF:
			// omit; presence is asserted separately
		default:
			out = append(out, tk.Kind.Description())
		}
	}
	return out
}

// mustTokenize tokenizes src and fails on any lexical error.
func mustTokenize(t *testing.T, src string) []token.Token {
	t.Helper()
	toks, errs := Tokenize([]byte(src))
	if len(errs) != 0 {
		t.Fatalf("unexpected lex errors for %q: %v", src, errs)
	}
	return toks
}

// body strips the trailing statement terminator and EOF so expectations can
// describe only the tokens that carry meaning. The lexer always appends
// NEWLINE + EOF, so keeping them in every expectation is noise.
func body(toks []token.Token) []token.Token {
	out := make([]token.Token, len(toks))
	copy(out, toks)
	if n := len(out); n > 0 && out[n-1].Kind == token.EOF {
		out = out[:n-1]
	}
	if n := len(out); n > 0 && out[n-1].Kind == token.NEWLINE {
		out = out[:n-1]
	}
	return out
}

func joinKinds(toks []token.Token) string { return strings.Join(kinds(toks), " ") }

func joinBody(toks []token.Token) string { return strings.Join(kinds(body(toks)), " ") }

func countKind(toks []token.Token, k token.Kind) int {
	n := 0
	for _, tk := range toks {
		if tk.Kind == k {
			n++
		}
	}
	return n
}

func TestTokenKindDescriptionsCoverAll(t *testing.T) {
	// Guards the description table against a silently unhandled new kind.
	for k := token.EOF; k <= token.RBRACK; k++ {
		if k.Description() == "UNKNOWN" {
			t.Errorf("kind %d has no description", int(k))
		}
	}
}

func TestPackedSourceTokenizesWithoutSpaces(t *testing.T) {
	// The core requirement: zero-whitespace packing must be unambiguous.
	src := `o=P.new(0,0);!v=V.new();v.push(Shape.C(3))`
	got := joinBody(mustTokenize(t, src))
	want := "IDENT(o) = TYPEIDENT(P) . IDENT(new) ( INT(0) , INT(0) ) ; ! IDENT(v) = TYPEIDENT(V) . IDENT(new) ( ) ; IDENT(v) . IDENT(push) ( TYPEIDENT(Shape) . TYPEIDENT(C) ( INT(3) ) )"
	if got != want {
		t.Errorf("packed source mismatch\n got: %s\nwant: %s", got, want)
	}
}

func TestSigilTable(t *testing.T) {
	cases := []struct {
		src  string
		want string
	}{
		{"=", "="},
		{"==", "=="},
		{"!=", "!="},
		{"<", "<"},
		{"<=", "<="},
		{">", ">"},
		{">=", ">="},
		{"!", "!"},
		{"?", "?"},
		{"~", "~"},
		{"@", "@"},
		{"#", "#"},
		{"^", "^"},
		{"|", "|"},
		{"|>", "|>"},
		{"=>", "=>"},
		{":", ":"},
		{",", ","},
		{".", "."},
		{";", ";"},
		{"&", "&"},
		{"+", "+"},
		{"-", "-"},
		{"*", "*"},
		{"/", "/"},
		{"(", "("},
		{")", ")"},
		{"{", "{"},
		{"}", "}"},
		{"[", "["},
		{"]", "]"},
		// "\" is intentionally absent: it is only valid as a line
		// continuation before a newline, covered separately below.
	}
	for _, c := range cases {
		toks := body(mustTokenize(t, c.src))
		if len(toks) != 1 {
			t.Errorf("src %q: got %d tokens, want 1 (%v)", c.src, len(toks), kinds(toks))
			continue
		}
		if got := toks[0].Kind.Description(); got != c.want {
			t.Errorf("src %q: got kind %q, want %q", c.src, got, c.want)
		}
	}
}

func TestMultiCharOperatorsNeverResplit(t *testing.T) {
	// Single-token multi-char operators are what let `a|>b` and `a=>b` read
	// as one thing each, so this pins the boundary against regressions.
	for _, c := range []struct{ src, want string }{
		{"a|>b", "IDENT(a) |> IDENT(b)"},
		{"a=>b", "IDENT(a) => IDENT(b)"},
		{"a<=b", "IDENT(a) <= IDENT(b)"},
		{"a>=b", "IDENT(a) >= IDENT(b)"},
		{"a!=b", "IDENT(a) != IDENT(b)"},
		{"a==b", "IDENT(a) == IDENT(b)"},
	} {
		got := joinBody(mustTokenize(t, c.src))
		if got != c.want {
			t.Errorf("src %q: got %q, want %q", c.src, got, c.want)
		}
	}
}

func TestReservedBoolConstantsAreNotIdentifiers(t *testing.T) {
	toks := body(mustTokenize(t, "t f"))
	if len(toks) != 2 || toks[0].Kind != token.BOOL || toks[0].Lit != "t" ||
		toks[1].Kind != token.BOOL || toks[1].Lit != "f" {
		t.Fatalf("t/f must lex as BOOL constants, got %v", kinds(toks))
	}
}

func TestIdentifiersAndTypeIdentifiers(t *testing.T) {
	toks := body(mustTokenize(t, `foo_bar a1 _leading Shape Vec`))
	wantID := map[string]bool{"foo_bar": true, "a1": true, "_leading": true}
	wantType := map[string]bool{"Shape": true, "Vec": true}
	for _, tk := range toks {
		switch tk.Kind {
		case token.IDENT:
			if !wantID[tk.Lit] {
				t.Errorf("unexpected IDENT %q", tk.Lit)
			}
		case token.TYPEIDENT:
			if !wantType[tk.Lit] {
				t.Errorf("unexpected TYPEIDENT %q", tk.Lit)
			}
		default:
			t.Errorf("unexpected token %s", tk)
		}
	}
}

func TestNumbers(t *testing.T) {
	for _, c := range []struct {
		src  string
		lit  string
		kind token.Kind
	}{
		{"0", "0", token.INT},
		{"42", "42", token.INT},
		{"1_000", "1_000", token.INT},
		{"3.14", "3.14", token.FLOAT},
		{"1_000.5", "1_000.5", token.FLOAT},
	} {
		toks := body(mustTokenize(t, c.src))
		if len(toks) != 1 || toks[0].Kind != c.kind || toks[0].Lit != c.lit {
			t.Errorf("src %q: got %v, want %s(%s)", c.src, kinds(toks), c.kind.Description(), c.lit)
		}
	}
}

func TestNumberTouchingIdentifierIsError(t *testing.T) {
	// 1abc would read two ways under packing; the lexer must reject it.
	_, errs := Tokenize([]byte("1abc"))
	if len(errs) != 1 {
		t.Fatalf("want exactly 1 error, got %v", errs)
	}
	if !strings.Contains(errs[0].Error(), "identifier character") {
		t.Errorf("unexpected error text: %v", errs[0])
	}
}

func TestDottedCallChainTokenizes(t *testing.T) {
	// Receiver-then-dot-then-method: the packed legal form of a method
	// call, and the shape every bench program uses constantly.
	toks := mustTokenize(t, "v.iter()")
	want := "IDENT(v) . IDENT(iter) ( )"
	if got := joinBody(toks); got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestStringsAndEscapes(t *testing.T) {
	toks := body(mustTokenize(t, `"a\nb\t\"c\\d\0e"`))
	if len(toks) != 1 || toks[0].Kind != token.STRING {
		t.Fatalf("got %v", kinds(toks))
	}
	want := "a\nb\t\"c\\d\x00e"
	if toks[0].Lit != want {
		t.Errorf("got %q, want %q", toks[0].Lit, want)
	}
}

func TestUnterminatedStringReportedAtOpeningQuote(t *testing.T) {
	_, errs := Tokenize([]byte("\"abc"))
	if len(errs) == 0 || errs[0].Line != 1 || errs[0].Col != 1 {
		t.Fatalf("want error at 1:1, got %v", errs)
	}
}

func TestNewlineTerminatesAndRunsCollapse(t *testing.T) {
	toks := mustTokenize(t, "a=1\n\n\n\nb=2")
	if n := countKind(body(toks), token.NEWLINE); n != 1 {
		t.Errorf("collapsed newline runs: got %d NEWLINE tokens, want 1", n)
	}
}

func TestLeadingNewlinesAreDropped(t *testing.T) {
	toks := mustTokenize(t, "\n\n\na=1")
	if toks[0].Kind == token.NEWLINE {
		t.Errorf("leading NEWLINE should be dropped, got %v", kinds(toks))
	}
}

func TestSemicolonIsAnEquivalentTerminator(t *testing.T) {
	// Semicolon and newline are the two accepted statement terminators:
	// lexically distinct tokens, but interchangeable once the parser
	// unifies them, so both streams are identical after that substitution.
	semi := mustTokenize(t, "a=1;b=2")
	nl := mustTokenize(t, "a=1\nb=2")
	normalized := func(toks []token.Token) string {
		var out []string
		for _, tk := range body(toks) {
			if tk.Kind == token.SEMICOLON {
				out = append(out, "NL")
				continue
			}
			switch tk.Kind {
			case token.NEWLINE:
				out = append(out, "NL")
			case token.EOF:
				// omit
			default:
				out = append(out, tk.String())
			}
		}
		return strings.Join(out, " ")
	}
	if got, want := normalized(semi), normalized(nl); got != want {
		t.Errorf("semicolon and newline should be interchangeable terminators:\n%s\n%s", got, want)
	}
	// And the semicolon form must actually carry a semicolon.
	if !containsKind(semi, token.SEMICOLON) {
		t.Errorf("expected a SEMICOLON token between statements, got %v", kinds(semi))
	}
}

func containsKind(toks []token.Token, k token.Kind) bool {
	for _, tk := range toks {
		if tk.Kind == k {
			return true
		}
	}
	return false
}

func TestCommentsRunToEndOfLine(t *testing.T) {
	toks := mustTokenize(t, "a=1 % this is a comment ? ! and junk\nb=2")
	if got := joinBody(toks); got != "IDENT(a) = INT(1) NL IDENT(b) = INT(2)" {
		t.Errorf("got %q", got)
	}
}

func TestCommentCannotHideTheStatementTerminator(t *testing.T) {
	// A comment consumes up to but not including the newline, so the
	// statement still ends.
	toks := mustTokenize(t, "a=1\n%\nb=2")
	if got, want := joinBody(toks), "IDENT(a) = INT(1) NL IDENT(b) = INT(2)"; got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestLineContinuationSwallowsTheNewline(t *testing.T) {
	toks := mustTokenize(t, "a=1+\\\n2")
	if n := countKind(body(toks), token.NEWLINE); n != 0 {
		t.Errorf("continued line should produce no NEWLINE, got %d", n)
	}
	if n := countKind(body(toks), token.CONTINUE); n != 1 {
		t.Errorf("want 1 CONTINUE token, got %d", n)
	}
}

func TestStrayBackslashIsAnError(t *testing.T) {
	_, errs := Tokenize([]byte("a=1 \\ 2"))
	if len(errs) != 1 {
		t.Fatalf("want 1 error, got %v", errs)
	}
	if !strings.Contains(errs[0].Error(), "line continuation") {
		t.Errorf("unexpected error text: %v", errs[0])
	}
}

func TestUnexpectedCharacterReported(t *testing.T) {
	_, errs := Tokenize([]byte("a=1 $ b=2"))
	if len(errs) != 1 {
		t.Fatalf("want 1 error, got %v", errs)
	}
	if !strings.Contains(errs[0].Error(), "unexpected character") {
		t.Errorf("unexpected error text: %v", errs[0])
	}
}

func TestPositionsAreByteExactUnderPacking(t *testing.T) {
	src := "abc=123"
	toks := mustTokenize(t, src)
	want := map[string]int{"abc": 1, "123": 5}
	for _, tk := range toks {
		if tk.Line != 1 {
			t.Fatalf("token %s: line %d, want 1", tk, tk.Line)
		}
		if w, ok := want[tk.Lit]; ok && tk.Col != w {
			t.Errorf("token %q at col %d, want %d", tk.Lit, tk.Col, w)
		}
	}
}

func TestStringTokenPositionIsTheOpeningQuote(t *testing.T) {
	toks := mustTokenize(t, `x="ab\ncd"`)
	var st token.Token
	for _, tk := range toks {
		if tk.Kind == token.STRING {
			st = tk
		}
	}
	if st.Lit == "" || st.Col != 3 {
		t.Errorf("string token %s at col %d, want STRING at col 3", st, st.Col)
	}
}

func TestEndsWithNewlineThenEOF(t *testing.T) {
	toks := mustTokenize(t, "a=1")
	last := toks[len(toks)-1]
	if last.Kind != token.EOF {
		t.Fatalf("last token %s, want EOF", last)
	}
	if prev := toks[len(toks)-2]; prev.Kind != token.NEWLINE {
		t.Errorf("token before EOF is %s, want NEWLINE", prev)
	}
}

func TestEmptySource(t *testing.T) {
	toks := mustTokenize(t, "")
	if len(toks) != 1 || toks[0].Kind != token.EOF {
		t.Fatalf("empty source should yield only EOF, got %v", kinds(toks))
	}
}

func TestSpecMinimalExampleTokenizes(t *testing.T) {
	// The minimal example from SPEC.md must tokenize end to end: it is the
	// contract every later parser test builds on.
	src := `T P { x:i, y:i }
# P {
  new(x:i, y:i) -> P { P{x:x, y:y} }
  add(&self, o:&P) -> P { P{x:self.x+o.x, y:self.y+o.y} }
}
E Shape { C(i), R(P, P) }
area(s:&Shape) -> i {
  #s { Shape.C(r)=>r*r*3, Shape.R(a,b)=>(b.x-a.x)*(b.y-a.y) }
}
main() {
  o=P.new(0,0)
  !v=V.new()
  v.push(Shape.C(3))
  v.iter()|s|{ print(area(s)) }
  ?o.x>5 {^1}
  0
}`
	if _, errs := Tokenize([]byte(src)); len(errs) != 0 {
		t.Fatalf("spec example should tokenize cleanly, got errors: %v", errs)
	}
}

func TestSpecPackedFormTokenizes(t *testing.T) {
	// The same program in the fully packed form the AI will actually emit.
	src := `T P{x:i,y:i}# P{new(x:i,y:i)->P{P{x:x,y:y}}add(&self,o:&P)->P{P{x:self.x+o.x,y:self.y+o.y}}}E Shape{C(i),R(P,P)}area(s:&Shape)i{#s{Shape.C(r)=>r*r*3,Shape.R(a,b)=>(b.x-a.x)*(b.y-a.y)}}main(){o=P.new(0,0);!v=V.new();v.push(Shape.C(3));v.iter()|s|{print(area(s))};?o.x>5{^1};0}`
	if _, errs := Tokenize([]byte(src)); len(errs) != 0 {
		t.Fatalf("packed spec example should tokenize cleanly, got errors: %v", errs)
	}
}
