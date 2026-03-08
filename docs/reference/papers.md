# Academic References

## Core ACO Papers

### Ant System (1996)

**M. Dorigo, V. Maniezzo, A. Colorni.** *Ant System: Optimization by a Colony of Cooperating Agents.* IEEE Transactions on Systems, Man, and Cybernetics — Part B, 26(1):29-41, 1996.

The foundational paper introducing ant-based optimization. Defines the core mechanics: probabilistic path construction using pheromone trails (tau) and heuristic information (eta), pheromone deposit by successful ants, and evaporation for forgetting.

**Key contribution**: Demonstrated that simple agents following local rules can solve NP-hard combinatorial optimization problems (TSP, QAP).

**REVM usage**: Core probability formula `P(i->j) = tau^alpha * eta^beta / sum` is directly from this paper.

---

### AntNet (1998)

**G. Di Caro, M. Dorigo.** *AntNet: Distributed Stigmergetic Control for Communications Networks.* Journal of Artificial Intelligence Research, 9:317-365, 1998.

Adapts Ant System to packet routing in telecommunications networks. Introduces:
- Mobile agents (ants) that explore the network in real-time
- Latency-based heuristic (eta = 1/delay)
- Adaptive pheromone update based on measured round-trip times
- Stochastic forwarding tables maintained by pheromone trails

**Key contribution**: Proved ACO works for dynamic, real-time network routing — not just static optimization problems.

**REVM usage**: The concept of routing ants through a live network topology with latency feedback comes directly from AntNet. REVM's latency probing and topology update mechanisms are inspired by AntNet's adaptive agents.

---

### MAX-MIN Ant System (2000)

**T. Stutzle, H.H. Hoos.** *MAX-MIN Ant System.* Future Generation Computer Systems, 16(8):889-914, 2000.

Introduces pheromone bounds to prevent premature convergence:
- `tau_max`: Upper bound prevents any edge from dominating
- `tau_min`: Lower bound ensures all edges remain explorable
- Only the iteration-best or global-best ant deposits pheromone
- Pheromone trails are re-initialized when stagnation is detected

**Key contribution**: MMAS consistently outperforms basic Ant System and most other ACO variants on benchmark problems. The bounds provide a principled way to balance exploration vs. exploitation.

**REVM usage**: `pheromone_min` and `pheromone_max` in `AcoConfig` implement MMAS bounds. After every evaporation and deposit cycle, pheromone values are clamped to `[pheromone_min, pheromone_max]`.

---

## Supplementary References

### ACO Metaheuristic Framework

**M. Dorigo, T. Stutzle.** *Ant Colony Optimization.* MIT Press, 2004.

Comprehensive textbook covering all ACO variants, convergence proofs, and practical implementation guidance.

### Swarm Intelligence

**M. Dorigo, M. Birattari, T. Stutzle.** *Ant Colony Optimization: Artificial Ants as a Computational Intelligence Technique.* IEEE Computational Intelligence Magazine, 1(4):28-39, 2006.

Survey of ACO applications across domains: routing, scheduling, assignment, and machine learning.

### Network Routing with ACO

**E. Bonabeau, M. Dorigo, G. Theraulaz.** *Swarm Intelligence: From Natural to Artificial Systems.* Oxford University Press, 1999.

Broader context of swarm intelligence in network optimization, with detailed analysis of AntNet performance vs. OSPF and other traditional routing protocols.

---

## Solana-Specific References

### Solana Architecture

**A. Yakovenko.** *Solana: A new architecture for a high performance blockchain.* Whitepaper, 2018.

Describes Proof of History, Tower BFT, and the slot-based leader rotation that REVM's routing strategies are built around.

### QUIC Transport

**J. Iyengar, M. Thomson.** *QUIC: A UDP-Based Multiplexed and Secure Transport.* RFC 9000, IETF, 2021.

The transport protocol used by Solana's TPU since v1.17. REVM's direct TPU send uses QUIC streams.

### Stake-Weighted QoS

**Solana Foundation.** *Stake-Weighted Quality of Service.* SIMD-0016, 2023.

Specification for stake-proportional QUIC connection allocation at validator TPU ports. Informs REVM's `StakeWeighted` routing strategy.
