// El contraste de la nota del medio (RFC-0013 E2b, §632) con una
// implementación ajena: filippo.io/torchwood v0.10.0 —la del autor de las
// especificaciones C2SP— sobre el crypto/mldsa de la biblioteca estándar de
// Go 1.27, y golang.org/x/mod/sumdb para la nota y el árbol.
//
//	go run . generar ..     escribe las notas y publicador.vkey en ..
//	go run . verificar ..   juzga cada nota de .. con note.Open y escribe
//	                        ../veredictos-torchwood.txt
//
// Nada de esto lo corre el canon: los ficheros que escribe se guardan en
// el árbol y los lee tests/vectores_notas.rs. Las semillas son de prueba.
package main

import (
	"bytes"
	"crypto"
	"crypto/mldsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"

	"filippo.io/torchwood"
	"golang.org/x/mod/sumdb/note"
	"golang.org/x/mod/sumdb/tlog"
)

// La huella de la clave XMSS del operador de spec/vectors/ancla/ (el campo
// `clave` de su ancla, RFC-0012 D-B): el medio de ese operador.
const huellaDelOperador = "8c40b55b1d40cf7f71f9ea99c770f701db95da5876e6ce889f86af12b0def53a"

const nombreTestigo = "testigo.invalid/ajeno"

// Firma determinista: misma entrada, misma firma, para que la otra
// implementación pueda reproducirla byte a byte con la misma marca.
type determinista struct{ k *mldsa.PrivateKey }

func (d determinista) Public() crypto.PublicKey { return d.k.Public() }
func (d determinista) Sign(_ io.Reader, m []byte, o crypto.SignerOpts) ([]byte, error) {
	return d.k.SignDeterministic(m, o)
}

func semilla(desde byte) []byte {
	s := make([]byte, 32)
	for i := range s {
		s[i] = desde + byte(i)
	}
	return s
}

func clave(desde byte) *mldsa.PrivateKey {
	k, err := mldsa.NewPrivateKey(mldsa.MLDSA44(), semilla(desde))
	if err != nil {
		panic(err)
	}
	return k
}

func firmante(nombre string, desde byte) *torchwood.CosignatureSigner {
	s, err := torchwood.NewCosignatureSigner(nombre, determinista{clave(desde)})
	if err != nil {
		panic(err)
	}
	return s
}

// La raíz del árbol de n anclas cuya huella i es SHA-256(u64be(i)), con el
// árbol de golang.org/x/mod/sumdb/tlog. El vacío es SHA-256 de nada, como
// en RFC 6962 y en tlog-checkpoint.
func raiz(n int) tlog.Hash {
	if n == 0 {
		return tlog.Hash(sha256.Sum256(nil))
	}
	var guardados []tlog.Hash
	lector := tlog.HashReaderFunc(func(idx []int64) ([]tlog.Hash, error) {
		out := make([]tlog.Hash, len(idx))
		for i, x := range idx {
			out[i] = guardados[x]
		}
		return out, nil
	})
	for i := 0; i < n; i++ {
		var b [8]byte
		binary.BigEndian.PutUint64(b[:], uint64(i))
		h := sha256.Sum256(b[:])
		hs, err := tlog.StoredHashes(int64(i), h[:], lector)
		if err != nil {
			panic(err)
		}
		guardados = append(guardados, hs...)
	}
	r, err := tlog.TreeHash(int64(n), lector)
	if err != nil {
		panic(err)
	}
	return r
}

func texto(origen string, n int, r tlog.Hash) string {
	return fmt.Sprintf("%s\n%d\n%s\n", origen, n, base64.StdEncoding.EncodeToString(r[:]))
}

func firmar(t string, s ...note.Signer) string {
	b, err := note.Sign(&note.Note{Text: t}, s...)
	if err != nil {
		panic(err)
	}
	return string(b)
}

// Las líneas de firma de una nota, y su texto con la línea en blanco.
func partir(nota string) (string, []string) {
	i := strings.LastIndex(nota, "\n\n")
	lineas := strings.Split(strings.TrimSuffix(nota[i+2:], "\n"), "\n")
	return nota[:i+2], lineas
}

func unir(cabeza string, lineas ...string) string {
	return cabeza + strings.Join(lineas, "\n") + "\n"
}

// Cambia los bytes de una línea de firma con f y la vuelve a escribir.
func tocar(linea string, f func([]byte) []byte) string {
	nombre, b64, _ := strings.Cut(strings.TrimPrefix(linea, "— "), " ")
	b, err := base64.StdEncoding.DecodeString(b64)
	if err != nil {
		panic(err)
	}
	return "— " + nombre + " " + base64.StdEncoding.EncodeToString(f(b))
}

// Pone a uno los bits sobrantes del último símbolo antes del relleno: la
// misma base64 para un decodificador laxo, otra cadena para uno estricto.
func noCanonica(b64 string) string {
	const alfabeto = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
	sin := strings.TrimRight(b64, "=")
	relleno := len(b64) - len(sin)
	if relleno == 0 {
		panic("sin bits sobrantes")
	}
	ultimo := strings.IndexByte(alfabeto, sin[len(sin)-1])
	ultimo |= 1
	return sin[:len(sin)-1] + string(alfabeto[ultimo]) + b64[len(sin):]
}

func generar(dir string) {
	huella, _ := hex.DecodeString(huellaDelOperador)
	origen := "zkssl/v1/" + hex.EncodeToString(huella)
	publicador := firmante(origen, 0x00)
	testigo := firmante(nombreTestigo, 0x20)
	escribir := func(nombre, contenido string) {
		if err := os.WriteFile(filepath.Join(dir, nombre), []byte(contenido), 0o644); err != nil {
			panic(err)
		}
	}
	escribir("publicador.vkey", publicador.Verifier().String()+"\n")
	escribir("testigo.vkey", testigo.Verifier().String()+"\n")

	// Positivas: el publicador y un testigo, sobre árboles de 0, 1, 2, 5 y 13.
	for _, n := range []int{0, 1, 2, 5, 13} {
		escribir(fmt.Sprintf("pos-%02d-anclas.txt", n), firmar(texto(origen, n, raiz(n)), publicador, testigo))
	}
	r5 := raiz(5)
	b64r5 := base64.StdEncoding.EncodeToString(r5[:])
	base := firmar(texto(origen, 5, r5), publicador, testigo)
	cabeza, lineas := partir(base)
	p, t := lineas[0], lineas[1]
	escribir("pos-sin-testigo.txt", unir(cabeza, p))
	escribir("pos-testigo-primero.txt", unir(cabeza, t, p))

	// Negativas, cada una de la positiva de 5 anclas.
	textoDe := func(s string) string { return strings.TrimSuffix(s, "\n") }
	neg := map[string]string{
		"neg-firma-cambiada.txt": unir(cabeza, tocar(p, func(b []byte) []byte { b[100] ^= 1; return b }), t),
		"neg-firma-truncada.txt": unir(cabeza, tocar(p, func(b []byte) []byte { return b[:len(b)-10] }), t),
		"neg-marca-cambiada.txt": unir(cabeza, tocar(p, func(b []byte) []byte { b[11] ^= 1; return b }), t),
		"neg-marca-2-63.txt": unir(cabeza, tocar(p, func(b []byte) []byte {
			binary.BigEndian.PutUint64(b[4:12], 1<<63)
			return b
		}), t),
		"neg-key-id-cambiado.txt":          unir(cabeza, tocar(p, func(b []byte) []byte { b[0] ^= 1; return b }), t),
		"neg-nombre-cambiado.txt":          unir(cabeza, strings.Replace(p, origen, origen+"x", 1), t),
		"neg-sin-firma-del-publicador.txt": unir(cabeza, t),
		"neg-raiz-cambiada.txt":            unir(textoDe(texto(origen, 5, raiz(4)))+"\n\n", p, t),
		"neg-tamano-cambiado.txt":          unir(textoDe(texto(origen, 6, raiz(5)))+"\n\n", p, t),
		"neg-origen-cambiado.txt":          unir(textoDe(texto("zkssl/v1/"+strings.Repeat("00", 32), 5, raiz(5)))+"\n\n", p, t),
		"neg-extension.txt":                unir(textoDe(texto(origen, 5, raiz(5)))+"\nextra\n\n", p, t),
		"neg-tamano-con-cero.txt":          unir(strings.Replace(cabeza, "\n5\n", "\n05\n", 1), p, t),
		"neg-raiz-no-canonica.txt":         unir(strings.Replace(cabeza, b64r5, noCanonica(b64r5), 1), p, t),
		"neg-sin-linea-en-blanco.txt":      strings.Replace(base, "\n\n", "\n", 1),
		"neg-dos-lineas-en-blanco.txt":     strings.Replace(base, "\n\n", "\n\n\n", 1),
		"neg-retorno-de-carro.txt":         strings.Replace(base, "\n5\n", "\n5\r\n", 1),
		"neg-linea-sin-raya.txt":           unir(cabeza, p, "- "+strings.TrimPrefix(t, "— ")),
		"neg-testigo-mal-formado.txt":      unir(cabeza, p, t+"!"),
		"neg-firma-repetida-identica.txt":  unir(cabeza, p, p, t),
		"neg-firma-repetida-y-basura.txt":  unir(cabeza, p, tocar(p, func(b []byte) []byte { b[100] ^= 1; return b }), t),
		"neg-firma-no-canonica.txt": unir(cabeza, func() string {
			nombre, b64, _ := strings.Cut(strings.TrimPrefix(p, "— "), " ")
			return "— " + nombre + " " + noCanonica(b64)
		}(), t),
	}
	// Otra clave con el mismo nombre: su key_id no es el del publicador.
	impostor := firmante(origen, 0x40)
	neg["neg-otra-clave-mismo-nombre.txt"] = firmar(texto(origen, 5, raiz(5)), impostor, testigo)
	// La clave del publicador firmando un checkpoint de OTRO origin: torchwood
	// lo firma y lo verifica; el medio exige origin = nombre (D-A).
	otro := "zkssl/v1/" + strings.Repeat("11", 32)
	neg["neg-origen-ajeno-firmado.txt"] = firmar(texto(otro, 5, raiz(5)), publicador, testigo)
	for nombre, contenido := range neg {
		escribir(nombre, contenido)
	}
}

func verificar(dir string) {
	vkey, err := os.ReadFile(filepath.Join(dir, "publicador.vkey"))
	if err != nil {
		panic(err)
	}
	v, err := torchwood.NewLogVerifier(strings.TrimSpace(string(vkey)))
	if err != nil {
		panic(err)
	}
	ficheros, _ := filepath.Glob(filepath.Join(dir, "*.txt"))
	sort.Strings(ficheros)
	var out bytes.Buffer
	fmt.Fprintf(&out, "# note.Open (golang.org/x/mod v0.40.0) con torchwood.NewLogVerifier (v0.10.0), Go %s\n", goVersion())
	for _, f := range ficheros {
		base := filepath.Base(f)
		if base == "veredictos-torchwood.txt" {
			continue
		}
		b, _ := os.ReadFile(f)
		n, err := note.Open(b, note.VerifierList(v))
		switch {
		case err != nil:
			fmt.Fprintf(&out, "%s RECHAZA %v\n", base, err)
		default:
			fmt.Fprintf(&out, "%s ACEPTA %d firma(s) verificada(s), %d ajena(s)\n", base, len(n.Sigs), len(n.UnverifiedSigs))
		}
	}
	if err := os.WriteFile(filepath.Join(dir, "veredictos-torchwood.txt"), out.Bytes(), 0o644); err != nil {
		panic(err)
	}
	os.Stdout.Write(out.Bytes())
}

func goVersion() string { return runtime.Version() }

func main() {
	if len(os.Args) != 3 {
		fmt.Fprintln(os.Stderr, "uso: go run . generar|verificar DIR")
		os.Exit(2)
	}
	switch os.Args[1] {
	case "generar":
		generar(os.Args[2])
	case "verificar":
		verificar(os.Args[2])
	default:
		os.Exit(2)
	}
}
