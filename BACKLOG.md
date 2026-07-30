# BACKLOG

Lo que falta, numerado y por prioridad. Existe para que las cosas puedan
citarse por su numero entre sesiones, igual que las secciones de
`AUDITORIA.md`.

**Los numeros no se reutilizan ni se renumeran.** Cuando algo se resuelve
se marca `[x]` y se anota con que commit, pero **se queda en la lista**.
Una lista que borra lo resuelto no deja ver que se ha hecho ni en que
orden; y este proyecto marca las correcciones en vez de borrarlas.

Lo que entre nuevo va al final con el numero siguiente, y se coloca en su
grupo de prioridad sin cambiar de numero.

**Estado**: 23 abiertas, 0 resueltas. Ultima revision: 30 de julio de 2026.

---

## A. Cerrar la obra en curso

Correcciones ya preparadas. Mientras no se apliquen, el repositorio
describe cosas que no son ciertas.

- [ ] **1. Coherencia de los papers.** `PAPER.md`, `PAPER_EN.md` y la
  documentacion autocontradictoria de `circuit_send` siguen describiendo
  el arbol de nullificadores como parte del estado. → `coherencia_papers.py`
- [ ] **2. Cifras del repositorio.** 65 de 174 en depuracion, 174 tests de
  la capa, doce circuitos, y la contradiccion 56/65 dentro de
  `AUDITORIA.md`. → `fix_cifras.py`
- [ ] **3. DOI de la tercera revision.** La seccion de publicacion apunta a
  versiones anteriores de los tres preprints. → `fix_dois.py`
- [ ] **4. Medir `stark-experiment` en depuracion.** El README publica
  «199 y 200» y en release hoy son 201; ultima cifra sin verificar.
- [ ] **5. Fuentes de los preprints al repositorio.** Los papeles dicen
  «reproducible desde el artefacto» y el texto publicado no esta en el
  artefacto.

## B. La siguiente de verdad

- [ ] **6. Reformular los grados del arbol de pendientes.** La unica
  comprobacion automatica del area declarada de menor confianza (§16.3)
  esta apagada, porque solo corre en depuracion y ahi fallan 65 tests
  (§20, §35).
- [ ] **7. Encargar la auditoria externa del argumento lockstep.** El
  hallazgo mas original del proyecto lo respalda un test discriminante, no
  una demostracion revisada por nadie mas (§16.4).

## C. Declaradas, acotadas, sin urgencia

- [ ] **8. `open_account` sin autorizacion.** El tope de cuentas mitiga a
  medias: un atacante aun puede agotar el cupo, y la solucion correcta
  exige un circuito nuevo (§16.1).
- [ ] **9. Congelacion sin caducidad ni motivo registrado.** El circuito
  prueba que dos custodios la autorizaron, no que tuvieran razon, y dura
  hasta que alguien la levante (§16.2).
- [ ] **10. Decidir sobre 127 bits conjeturados frente a 29–63
  demostrables.** El coste de cerrar la brecha esta medido
  (36,7 KB → 125,6 KB); falta hacer la eleccion explicita.
- [ ] **11. Canal lateral de ISO 20022.** Posicion, aleatorio e importe del
  pendiente viajan fuera del mensaje, sin especificar como; bloquea
  cualquier piloto.
- [ ] **12. Sin devolucion para un pendiente no cobrado.** El importe queda
  inmovilizado y no hay camino de vuelta implementado.
- [ ] **13. Senal temporal para el pagador.** Puede recomputar el
  compromiso y ver cuando se cobra; declarado, no eliminado.
- [ ] **14. Techo de 128 custodios.** Acoplado a un segmento de rango de 7
  bits, con holgura hoy (el arbol admite 16) pero sin declarar hasta que un
  test lo fijo.
- [ ] **15. Goldilocks es estrecho para identidades.** 64 bits son colision
  en 2³².
- [ ] **16. Referencias cruzadas de los preprints.** Los tres citan
  versiones anteriores de sus companeros; primera cosa de la cuarta
  revision.

## D. Operacion

- [ ] **17. Replica y alta disponibilidad.** El nodo es punto unico de
  fallo.
- [ ] **18. Bloqueo de directorio de `sled`.** Puede impedir un reinicio
  inmediato tras cerrar (§16.6).
- [ ] **19. Sin log de escritura anticipada.** Un fallo entre operaciones
  detiene el arranque pidiendo intervencion manual: correcto, pero no
  automatico.
- [ ] **20. Rotacion de claves operativas.** Implementada solo en parte.
- [ ] **21. Delegacion de prueba a clientes ligeros.** Exige verificar una
  firma dentro del circuito, ~8.000 filas.
- [ ] **22. Agregacion de pruebas.** 120,4 MB por mil pagos es coste, no
  parada, pero crece linealmente.

## E. Otro proyecto, no una incidencia

- [ ] **23. Consenso distribuido.** No anade un problema nuevo: recupera el
  del doble gasto que se cerro, y con el el limite del cumpleanos, salvo
  que se indexe por el nullificador completo (§13, §32, §36).

---

## Lo que NO esta en esta lista

Porque esta construido y medido: conservacion del valor, autoridad de
gasto sin entregar claves, emision con umbral, congelacion impuesta en
circuito, revelacion selectiva en tres modos, registro encadenado de
transiciones, persistencia e instantaneas con verificacion de integridad,
y cifrado en reposo.

Y una advertencia que pertenece aqui tanto como los pendientes: **esta
lista no se termina.** Cada cosa que se cierre destapara otras — ha pasado
tres veces esta semana. El valor del proyecto no esta en llegar a cero
pendientes, sino en saber con precision que es y que no es.
