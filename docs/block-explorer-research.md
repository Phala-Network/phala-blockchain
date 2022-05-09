Research: Open Source Substrate Blockchain Explorer Compatible With Phala Network
=================================================================================

This research is not intended to diminish the work of the teams or
contributors to the particular Block Explorer project. It is conducted
for research purposes only.

List of candidates
------------------

The Dotscanner block explorer will not be analyzed any further, since it
is not an open source solution.

<table>
<colgroup>
<col style="width: 9%" />
<col style="width: 46%" />
<col style="width: 29%" />
<col style="width: 14%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Description</th>
<th style="text-align: left;">Github</th>
<th style="text-align: left;">Explorer URL</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">Blockchain explorer for Polkadot, Kusama, and other related chains</td>
<td style="text-align: left;"><a href="https://github.com/polkascan/polkascan-os" class="uri">https://github.com/polkascan/polkascan-os</a></td>
<td style="text-align: left;"><a href="https://polkascan.io/" class="uri">https://polkascan.io/</a></td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">Blockchain explorer for Substrate chains</td>
<td style="text-align: left;"><a href="https://github.com/subscan-explorer/subscan-essentials" class="uri">https://github.com/subscan-explorer/subscan-essentials</a></td>
<td style="text-align: left;"><a href="https://www.subscan.io/" class="uri">https://www.subscan.io/</a></td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">Polkadot network statistics</td>
<td style="text-align: left;"><a href="https://github.com/orgs/Colm3na/repositories?q=polkastats&amp;type=all&amp;language=&amp;sort=" class="uri">https://github.com/orgs/Colm3na/repositories?q=polkastats&amp;type=all&amp;language=&amp;sort=</a></td>
<td style="text-align: left;"><a href="https://polkastats.io/" class="uri">https://polkastats.io/</a></td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">Polkadot dashboard block explorer supports dozens of other networks, including Kusama, Westend, and other remote or local endpoint</td>
<td style="text-align: left;"><a href="https://github.com/polkadot-js/apps" class="uri">https://github.com/polkadot-js/apps</a></td>
<td style="text-align: left;"><a href="https://polkadot.js.org/apps/#/explorer" class="uri">https://polkadot.js.org/apps/#/explorer</a></td>
</tr>
<tr class="odd">
<td style="text-align: left;">Dotscanner</td>
<td style="text-align: left;">is a next-generation blockchain explorer for Polkadot and other substrate-based networks, including Kusama</td>
<td style="text-align: left;">—</td>
<td style="text-align: left;"><a href="https://dotscanner.com/" class="uri">https://dotscanner.com/</a></td>
</tr>
</tbody>
</table>

Tech stack
----------

| Explorer                  | Backend            | Frontend | Database        |
|:--------------------------|:-------------------|:---------|:----------------|
| Polkascan                 | Python (Substrate) | Angular  | MySQL           |
| Subscan                   | Go                 | Vue.js   | MySQL           |
| Polkastats                | Node.js            | Vue.js   | PostgresSQL     |
| Polkadot-JS Apps Explorer | TypeScript         | React.js | Access via IPFS |

Community
---------

| Explorer                  | Open Source | License                         | Actively mantained |  Stars|  Forks|
|:--------------------------|:------------|:--------------------------------|:-------------------|------:|------:|
| Polkascan                 | yes         | GNU General Public License v3.0 | yes                |     51|     53|
| Subscan                   | yes         | GNU General Public License v3.0 | yes                |    131|     79|
| Polkastats                | yes         | GNU General Public License v3.0 | yes                |     21|     25|
| Polkadot-JS Apps Explorer | yes         | Apache License 2.0              | yes                |   1712|    829|

Code quality
------------

<table>
<colgroup>
<col style="width: 21%" />
<col style="width: 13%" />
<col style="width: 10%" />
<col style="width: 12%" />
<col style="width: 8%" />
<col style="width: 13%" />
<col style="width: 20%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Overall feeling</th>
<th style="text-align: left;">Code quality</th>
<th style="text-align: left;">Documentations</th>
<th style="text-align: left;">Unit tests</th>
<th style="text-align: left;">Functional tests</th>
<th style="text-align: right;">Code coverage in percent</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">ok</td>
<td style="text-align: left;">good</td>
<td style="text-align: left;">well</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: right;">88.00</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">very good</td>
<td style="text-align: left;">good</td>
<td style="text-align: left;">very well</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: right;">93.80</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">ok</td>
<td style="text-align: left;">ok</td>
<td style="text-align: left;">basic</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
<td style="text-align: right;">0.00</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">very good</td>
<td style="text-align: left;">good</td>
<td style="text-align: left;">very well</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: right;">87.83</td>
</tr>
</tbody>
</table>

Devops
------

<table>
<colgroup>
<col style="width: 15%" />
<col style="width: 27%" />
<col style="width: 34%" />
<col style="width: 21%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Docker Compose or similar deployment options</th>
<th style="text-align: left;">Single deployment for multichain or multiple deployment</th>
<th style="text-align: left;">Telemetry and monitoring mechanism</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">docker compose</td>
<td style="text-align: left;">multiple</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">docker compose</td>
<td style="text-align: left;">single</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">docker compose</td>
<td style="text-align: left;">single</td>
<td style="text-align: left;">x</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">docker build</td>
<td style="text-align: left;">single</td>
<td style="text-align: left;">x</td>
</tr>
</tbody>
</table>

Features
--------

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 23%" />
<col style="width: 14%" />
<col style="width: 11%" />
<col style="width: 10%" />
<col style="width: 11%" />
<col style="width: 8%" />
<col style="width: 8%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Extensible to non Polkadot Kusama blockchains</th>
<th style="text-align: left;">Custom JSON type definition</th>
<th style="text-align: left;">Governance info module</th>
<th style="text-align: left;">Staking info module</th>
<th style="text-align: left;">Custom module support</th>
<th style="text-align: left;">Frontend design</th>
<th style="text-align: left;">Mobile friendly</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">smooth</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">smooth</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">basic</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">basic</td>
<td style="text-align: left;">✔</td>
</tr>
</tbody>
</table>

### Transaction features

<table style="width:100%;">
<colgroup>
<col style="width: 19%" />
<col style="width: 12%" />
<col style="width: 18%" />
<col style="width: 33%" />
<col style="width: 15%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Transfer history</th>
<th style="text-align: left;">Non standard extrinsics</th>
<th style="text-align: left;">Filterable by pallet extrinsic name account</th>
<th style="text-align: left;">Searchable by fields</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">unknown</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">unknown</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">unknown</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">unknown</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
</tr>
</tbody>
</table>

### Event features

<table>
<colgroup>
<col style="width: 16%" />
<col style="width: 14%" />
<col style="width: 24%" />
<col style="width: 30%" />
<col style="width: 13%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Event by block number</th>
<th style="text-align: left;">Non standard events details decoding</th>
<th style="text-align: left;">Filterable by pallet event name caller account</th>
<th style="text-align: left;">Searchable by fields</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
</tr>
</tbody>
</table>

### Statistics features

<table>
<colgroup>
<col style="width: 18%" />
<col style="width: 31%" />
<col style="width: 24%" />
<col style="width: 25%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Explorer</th>
<th style="text-align: left;">Tokenomic metrics total supply staking etc</th>
<th style="text-align: left;">Extrinsics by pallet name account</th>
<th style="text-align: left;">Event by pallet name caller account</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;">Polkascan</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Subscan</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
</tr>
<tr class="odd">
<td style="text-align: left;">Polkastats</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">✔</td>
</tr>
<tr class="even">
<td style="text-align: left;">Polkadot-JS Apps Explorer</td>
<td style="text-align: left;">✔</td>
<td style="text-align: left;">x</td>
<td style="text-align: left;">x</td>
</tr>
</tbody>
</table>

Conclusion
----------

SUBSCAN appears to be the block explorer into which the Phala network
can be integrated. It is well documented and maintained, has a high code
quality and meets almost all features of the evaluation.

Deployment Guide
----------------

The network will be included by SUBSCAN. Therefore a form has to be
filled.

**<a href="https://subscan.typeform.com/to/jqdAWJ?typeform-source=www.subscan.io" class="uri">https://subscan.typeform.com/to/jqdAWJ?typeform-source=www.subscan.io</a>**
