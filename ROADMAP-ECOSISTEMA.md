# Roadmap de ecosistema — qué convierte una implementación en un estándar

Ni SAP ni Ethereum ganaron por la calidad del núcleo solamente: ganaron
porque **cualquiera podía construir encima sin pedir permiso**. Eso lo
dan cuatro cosas: una especificación estable, una implementación de
referencia, SDKs que preservan el modelo de seguridad, y vectores de
conformidad para que existan segundas implementaciones. Este roadmap
adapta las herramientas más aceptadas de otros ecosistemas a ZK-SSL.

## Mapa: herramienta consagrada → equivalente ZK-SSL

| ecosistema | herramienta | equivalente ZK-SSL | estado |
|---|---|---|---|
| Ethereum | `execution-apis` (spec RPC) | `spec/RPC.md` (`zkssl/0.1`) | **este paquete** |
| Ethereum | convención QUANTITY/DATA | `zk-ssl-wire` (hex 0x, digests canónicos) | **este paquete** |
| Ethereum | geth (nodo) | `zk-ssl-node` | **este paquete** |
| Foundry | anvil (devnet + faucet) | `zk-ssl-node --dev` (`dev_fund` con nullifiers de custodio visibles) | **este paquete** |
| Ethereum | ethers/web3 (SDK) | `zk-ssl-sdk` (prueba LOCAL, la clave no viaja) | **este paquete** |
| Foundry | cast / consola | `zk-ssl-cli` (simulate / trace-tx / inspect-state) | hecho (turno anterior) |
| Ethereum | OpenRPC JSON | generado desde `zk-ssl-wire` | siguiente |
| Ethereum | EIPs | proceso RFC del protocolo (plantilla + numeración) | siguiente |
| Ethereum | hive / test vectors | **vectores de conformidad**: escenario canónico → raíces y digests esperados por versión | siguiente |
| Ethereum | Etherscan | explorador sobre `zkssl_logEntries` + `verifyChain` (HTML estático primero) | siguiente |
| geth | keystore cifrado | wallet en reposo con chacha20poly1305 + KDF (deps ya presentes en zk-ssl) | siguiente |
| — | TS SDK | cliente TypeScript generado contra `spec/RPC.md` (probar en navegador exige compilar el prover a WASM: evaluar) | después |
| **SAP** | hablar el idioma de la industria | **ISO 20022**: el puente ya existe (`iso-bridge`); certificar mensajes pacs/camt contra la capa es el movimiento tipo SAP, no el tipo Ethereum | después, prioritario |

## Fases

**Fase 0 — este paquete.** Wire + spec + nodo de referencia + SDK.
Criterio de éxito: `zk-ssl-sdk` completa un pago en dos fases contra
`zk-ssl-node --dev` sin que ninguna clave de gasto viaje.

**Fase 1 — segundas implementaciones posibles.** OpenRPC generado,
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
