// Command elang is the compiler CLI for the elang language.
//
// elang source is a dense symbol stream compiled to native binaries through
// LLVM. See SPEC.md for the language definition.
package main

import (
	"fmt"
	"os"
)

const version = "0.1.0-dev"

func main() {
	if err := run(os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, "elang:", err)
		os.Exit(1)
	}
}

func run(args []string) error {
	if len(args) == 0 {
		usage()
		return nil
	}
	switch args[0] {
	case "version", "-v", "--version":
		fmt.Printf("elang %s\n", version)
	case "help", "-h", "--help":
		usage()
	case "build":
		return notImplemented("elang build <file.el>")
	case "fmt":
		return notImplemented("elang fmt <file.el>")
	case "ir":
		return notImplemented("elang ir <file.el>")
	case "obj":
		return notImplemented("elang obj <file.el>")
	case "bench":
		return notImplemented("elang bench")
	default:
		return fmt.Errorf("unknown command %q (try `elang help`)", args[0])
	}
	return nil
}

func usage() {
	fmt.Print(`elang - AI-first systems language

Usage:
  elang <command> [arguments]

Commands:
  build <file.el>   compile to a native binary
  fmt   <file.el>   canonicalize spelling (only ever removes characters)
  ir    <file.el>   print LLVM IR
  obj   <file.el>   print object dump
  bench             characters-per-program benchmark suite
  version           print version
  help              print this help

Docs: SPEC.md
`)
}

func notImplemented(form string) error {
	return fmt.Errorf("`%s` is not implemented yet", form)
}
