// Los testigos LOCALES del banco del publicador del medio (RFC-0013 E3a,
// §808): el motor de litewitness con una clave ML-DSA-44, y el agente SSH en
// memoria que el litewitness sin tocar necesita para su clave Ed25519.
//
// ⚠️ No compila aquí: se compila DENTRO de una copia de
// filippo.io/torchwood v0.10.0, como su cmd/testigo-local, porque usa
// filippo.io/torchwood/internal/witness —el paquete con el que litewitness
// atiende add-checkpoint—, y Go solo deja importar un internal desde su
// propio módulo. ../rehacer.sh
// (crates/zk-ssl-medio/tests/vectores/testigos/rehacer.sh) hace la copia,
// deja este fichero en ella y la borra al salir.
//
//	testigo-local mldsa  -name N -seed HEX -db F -listen A
//	    el add-checkpoint de litewitness (internal/witness.NewWitness, el
//	    mismo MaxBytesHandler de 10 KiB y los mismos plazos), con una clave
//	    ML-DSA-44 de esa semilla en lugar de la Ed25519 del agente SSH.
//	    Escribe su vkey y su dirección, una por línea, y atiende.
//	testigo-local agente -seed HEX -sock S
//	    un ssh-agent en memoria con la Ed25519 de esa semilla. Escribe la
//	    huella SHA256: que litewitness pide en -key, y atiende.
//	testigo-local comprobar -vkey V
//	    lee una nota cofirmada por la entrada estándar y dice si la
//	    cofirma de esa vkey verifica, con el verificador de torchwood.
//
// Las semillas son de prueba. Nada de esto lo corre el canon: el banco
// guarda lo que contestan los testigos y tests/testigos_torchwood.rs lo
// relee.
package main

import (
	"crypto/ed25519"
	"crypto/mldsa"
	"encoding/hex"
	"flag"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"os"
	"time"

	"golang.org/x/crypto/ssh"
	"golang.org/x/crypto/ssh/agent"
	"golang.org/x/mod/sumdb/note"

	"filippo.io/torchwood"
	"filippo.io/torchwood/internal/witness"
)

func uso() {
	fmt.Fprintln(os.Stderr, "uso: testigo-local mldsa|agente|comprobar [opciones]")
	os.Exit(2)
}

func fatal(que string, err error) {
	fmt.Fprintf(os.Stderr, "testigo-local: %s: %v\n", que, err)
	os.Exit(1)
}

func semilla(h string) []byte {
	s, err := hex.DecodeString(h)
	if err != nil || len(s) != 32 {
		fatal("semilla", fmt.Errorf("32 bytes en hexadecimal"))
	}
	return s
}

func main() {
	if len(os.Args) < 2 {
		uso()
	}
	fs := flag.NewFlagSet(os.Args[1], flag.ExitOnError)
	switch os.Args[1] {
	case "mldsa":
		nombre := fs.String("name", "", "nombre del testigo")
		s := fs.String("seed", "", "semilla ML-DSA-44, 32 bytes en hex")
		db := fs.String("db", "", "base de datos sqlite (la de witnessctl)")
		escucha := fs.String("listen", "127.0.0.1:0", "dirección local")
		fs.Parse(os.Args[2:])
		k, err := mldsa.NewPrivateKey(mldsa.MLDSA44(), semilla(*s))
		if err != nil {
			fatal("clave", err)
		}
		w, err := witness.NewWitness(*db, *nombre, k, slog.New(slog.NewTextHandler(os.Stderr, nil)))
		if err != nil {
			fatal("testigo", err)
		}
		ln, err := net.Listen("tcp", *escucha)
		if err != nil {
			fatal("escucha", err)
		}
		fmt.Println(w.VerifierKey())
		fmt.Println(ln.Addr().String())
		// Lo mismo que litewitness pone delante de su mux.
		srv := &http.Server{
			Handler:      http.MaxBytesHandler(w, 10*1024),
			ReadTimeout:  5 * time.Second,
			WriteTimeout: 5 * time.Second,
		}
		fatal("servidor", srv.Serve(ln))
	case "agente":
		s := fs.String("seed", "", "semilla Ed25519, 32 bytes en hex")
		sock := fs.String("sock", "", "socket unix del agente")
		fs.Parse(os.Args[2:])
		k := ed25519.NewKeyFromSeed(semilla(*s))
		kr := agent.NewKeyring()
		if err := kr.Add(agent.AddedKey{PrivateKey: k}); err != nil {
			fatal("agente", err)
		}
		pk, err := ssh.NewPublicKey(k.Public())
		if err != nil {
			fatal("clave", err)
		}
		ln, err := net.Listen("unix", *sock)
		if err != nil {
			fatal("socket", err)
		}
		fmt.Println(ssh.FingerprintSHA256(pk))
		for {
			c, err := ln.Accept()
			if err != nil {
				fatal("socket", err)
			}
			go agent.ServeAgent(kr, c)
		}
	case "comprobar":
		vkey := fs.String("vkey", "", "vkey del testigo")
		fs.Parse(os.Args[2:])
		v, err := torchwood.NewCosignatureVerifier(*vkey)
		if err != nil {
			fatal("vkey", err)
		}
		nota, err := io.ReadAll(os.Stdin)
		if err != nil {
			fatal("entrada", err)
		}
		n, err := note.Open(nota, note.VerifierList(v))
		if err != nil {
			fatal("la cofirma no verifica", err)
		}
		t, err := torchwood.CosignatureTimestamp(n.Sigs[0])
		if err != nil {
			fatal("marca", err)
		}
		fmt.Printf("VERIFICA %s %d\n", v.Name(), t)
	default:
		uso()
	}
}
