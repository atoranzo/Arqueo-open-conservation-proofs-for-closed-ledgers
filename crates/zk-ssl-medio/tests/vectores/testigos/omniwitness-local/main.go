// Las claves y la comprobación del contraste del publicador del medio con
// omniwitness (RFC-0013 E3a, §848): el testigo de
// github.com/transparency-dev/witness en 5397155, que firma con las claves
// de github.com/transparency-dev/formats en 340a9e5.
//
// ⚠️ No compila aquí: se compila DENTRO de una copia de
// github.com/transparency-dev/witness en 5397155, como su cmd/omniwitness-local,
// para usar las versiones de formats y de filippo.io/mldsa que fija su
// go.mod. ../rehacer_omniwitness.sh
// (crates/zk-ssl-medio/tests/vectores/testigos/rehacer_omniwitness.sh) hace
// la copia, deja este fichero en ella y la borra al salir.
//
//	omniwitness-local clave -alg mldsa|ed25519 -name N -seed HEX -out F
//	    escribe en F la clave privada que omniwitness lee con
//	    --private_key_path (PRIVATE+KEY+N+key_id+base64), de tipo 0x06
//	    (ML-DSA-44, la semilla de FIPS 204) o 0x04 (Ed25519, la semilla de
//	    RFC 8032), y escribe por la salida estándar su vkey.
//	omniwitness-local comprobar -vkey V
//	    lee una nota cofirmada por la entrada estándar y dice si la
//	    cofirma de esa vkey verifica, con el verificador de formats que usa
//	    omniwitness.
//
// Las semillas son de prueba. Nada de esto lo corre el canon: el banco
// guarda lo que contesta omniwitness y tests/testigos_omniwitness.rs lo
// relee.
package main

import (
	"crypto/ed25519"
	"encoding/base64"
	"encoding/hex"
	"flag"
	"fmt"
	"io"
	"os"

	"filippo.io/mldsa"
	f_note "github.com/transparency-dev/formats/note"
	"golang.org/x/mod/sumdb/note"
)

const (
	tipoEd25519Cofirma = 0x04
	tipoMLDSA44        = 0x06
)

func uso() {
	fmt.Fprintln(os.Stderr, "uso: omniwitness-local clave|comprobar [opciones]")
	os.Exit(2)
}

func fatal(que string, err error) {
	fmt.Fprintf(os.Stderr, "omniwitness-local: %s: %v\n", que, err)
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
	case "clave":
		alg := fs.String("alg", "mldsa", "mldsa o ed25519")
		nombre := fs.String("name", "", "nombre del testigo")
		s := fs.String("seed", "", "semilla, 32 bytes en hex")
		salida := fs.String("out", "", "fichero de la clave privada")
		fs.Parse(os.Args[2:])
		sem := semilla(*s)
		var tipo byte
		var publica []byte
		switch *alg {
		case "mldsa":
			k, err := mldsa.NewPrivateKey(mldsa.MLDSA44(), sem)
			if err != nil {
				fatal("clave", err)
			}
			tipo, publica = tipoMLDSA44, k.PublicKey().Bytes()
		case "ed25519":
			tipo = tipoEd25519Cofirma
			publica = ed25519.NewKeyFromSeed(sem).Public().(ed25519.PublicKey)
		default:
			uso()
		}
		b64 := base64.StdEncoding.EncodeToString(append([]byte{tipo}, sem...))
		// El key_id de la clave privada no lo comprueba formats: se lee del
		// firmante que sale de ella, y se escribe el bueno.
		provisional, err := f_note.NewSignerForCosignatureV1(
			fmt.Sprintf("PRIVATE+KEY+%s+%08x+%s", *nombre, 0, b64))
		if err != nil {
			fatal("firmante", err)
		}
		id := provisional.KeyHash()
		skey := fmt.Sprintf("PRIVATE+KEY+%s+%08x+%s", *nombre, id, b64)
		vkey := fmt.Sprintf("%s+%08x+%s", *nombre, id,
			base64.StdEncoding.EncodeToString(append([]byte{tipo}, publica...)))
		// La vkey tiene que dar el mismo key_id con el verificador de formats.
		v, err := f_note.NewVerifierForCosignatureV1(vkey)
		if err != nil || v.KeyHash() != id {
			fatal("vkey", fmt.Errorf("el key_id no cuadra"))
		}
		if err := os.WriteFile(*salida, []byte(skey), 0o600); err != nil {
			fatal("fichero", err)
		}
		fmt.Println(vkey)
	case "comprobar":
		vkey := fs.String("vkey", "", "vkey del testigo")
		fs.Parse(os.Args[2:])
		v, err := f_note.NewVerifierForCosignatureV1(*vkey)
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
		t, err := f_note.CosignatureTimestamp(n.Sigs[0])
		if err != nil {
			fatal("marca", err)
		}
		fmt.Printf("VERIFICA %s %d\n", v.Name(), t)
	default:
		uso()
	}
}
