# Roadmap de ecosistema — qué convierte una implementación en un estándar

Ni SAP ni Ethereum ganaron por la calidad del núcleo solamente: ganaron
porque **cualquiera podía construir encima sin pedir permiso**. Eso lo
dan cuatro cosas: una especificación estable, una implementación de
referencia, SDKs que preservan el modelo de seguridad, y vectores de
conformidad para que existan segundas implementaciones. Este roadmap
adapta las herramientas más aceptadas de otros ecosistemas a ZK-SSL.

> **Estado (06-08-2026): las Fases 0 y 1 están EJECUTADAS y selladas**
> (§197–§199 de `AUDITORIA.md`). Este documento pasó de plan a acta;
> la Fase 2 queda abierta.

## Mapa: herramienta consagrada → equivalente ZK-SSL

| ecosistema | herramienta | equivalente ZK-SSL | estado |
|---|---|---|---|
| Ethereum | `execution-apis` (spec RPC) | `spec/RPC.md` (**`zkssl/0.3`** desde §354) | ✅ sellado (§197) |
| Ethereum | convención QUANTITY/DATA | `zk-ssl-wire` (hex 0x, digests canónicos) | ✅ sellado (§197) |
| Ethereum | geth (nodo) | `zk-ssl-node` | ✅ sellado (§197) |
| Foundry | anvil (devnet + faucet) | `zk-ssl-node --dev` (`dev_fund` con nullifiers de custodio visibles) | ✅ sellado (§197) |
| Ethereum | ethers/web3 (SDK) | `zk-ssl-sdk` (prueba LOCAL, la clave no viaja) | ✅ sellado (§197) |
| Foundry | cast / consola | `zk-ssl-cli` (simulate / trace-tx / inspect-state / **conformance**) | ✅ sellado (§197–§198) |
| Ethereum | OpenRPC JSON | generado desde `zk-ssl-wire` | ✅ sellado (§198): regenerar reproduce byte a byte |
| Ethereum | EIPs | proceso RFC del protocolo (plantilla + numeración) | ✅ sellado (§198): `spec/rfc/` |
| Ethereum | hive / test vectors | **vectores de conformidad**: escenario canónico → raíces y digests esperados por versión | ✅ sellado (§198): `conformance --check`, compuerta permanente |
| Ethereum | Etherscan | explorador sobre `zkssl_logEntries` + `verifyChain` (HTML estático primero) | Fase 2 |
| geth | keystore cifrado | wallet en reposo con chacha20poly1305 + KDF (deps ya presentes en zk-ssl) | ✅ sellado (§199): dominio propio, test de dominios |
| — | TS SDK | cliente TypeScript generado contra `spec/RPC.md` (probar en navegador exige compilar el prover a WASM: evaluar) | después |
| **SAP** | hablar el idioma de la industria | **ISO 20022**: el puente ya existe (`iso-bridge`); certificar mensajes pacs/camt contra la capa es el movimiento tipo SAP, no el tipo Ethereum | después, prioritario |

## Fases

**Fase 0 — ✅ SELLADA (§197).** Wire + spec + nodo de referencia + SDK.
Criterio de éxito CUMPLIDO y actado: el e2e terminó con alice 750000 ·
bob 1250000 · `verifyChain ok`, **sin que ninguna clave de gasto viaje**
(verificado en código: `spend_key` ni siquiera implementa `Serialize`).

**Fase 1 — ✅ SELLADA (§198–§199).** OpenRPC generado,
vectores de conformidad versionados (las raíces son deterministas por
operación; los digests de prueba se fijan POR VERSIÓN de circuito),
proceso RFC, keystore.

**Fase 2 — adopción.** Explorador, TS SDK, y la vía institucional:
conformidad ISO 20022 extremo a extremo con `iso-bridge`, que es donde
este sistema compite de verdad (liquidación), no en el nicho de Ethereum.

## Deudas conocidas que el RPC hereda (declaradas, no escondidas)

- El aviso (`PendingNotice`) viaja **fuera de banda** (§21): el RPC no
  lo transporta a terceros a propósito, pero el problema de transporte
  pagador→receptor sigue abierto.
- Nodo único: el operador ordena y puede censurar; el RPC no lo oculta.
- `--dev` usa custodios de prueba; un build de producción se compila sin
  esa feature y hoy exige añadir flags para raíces reales (marcado en
  `zk-ssl-node/src/main.rs`).
