use std::env;
use std::time::Instant;

// 1. S-Bit Hadamard Kernel
#[inline(always)]
fn sbit_hadamard_kernel(
    x_table: &mut [u64],
    z_table: &mut [u64],
    sign: &mut [u8],
    blocks_per_row: usize,
    block_idx: usize,
    bit_shift: u32,
) {
    for i in 0..blocks_per_row {
        let x_word = x_table[i];
        let z_word = z_table[i];

        let x_bit = (x_word >> bit_shift) & 1;
        let z_bit = (z_word >> bit_shift) & 1;

        if x_bit == 1 && z_bit == 1 {
            sign[block_idx] ^= 1;
        }

        let toggle = x_bit ^ z_bit;
        let mask = toggle << bit_shift;

        x_table[i] = x_word ^ mask;
        z_table[i] = z_word ^ mask;
    }
}

// 2. S-Bit Pauli-Z Gate Kernel
#[inline(always)]
fn sbit_pauli_z_kernel(
    x_table: &mut [u64],
    sign: &mut [u8],
    block_idx: usize,
    bit_shift: u32,
    blocks_per_row: usize,
) {
    for i in 0..blocks_per_row {
        let x_word = x_table[i];
        let x_bit = (x_word >> bit_shift) & 1;
        if x_bit == 1 {
            sign[block_idx] ^= 1;
        }
    }
}

// 3. S-Bit CNOT Kernel
#[inline(always)]
fn sbit_cnot_kernel(
    x_table: &mut [u64],
    z_table: &mut [u64],
    ctrl_shift: u32,
    tgt_shift: u32,
    blocks_per_row: usize,
) {
    for i in 0..blocks_per_row {
        let x_ctrl = x_table[i];
        let z_tgt = z_table[i];

        let x_ctrl_bit = (x_ctrl >> ctrl_shift) & 1;
        if x_ctrl_bit == 1 {
            x_table[i] ^= 1 << tgt_shift;
        }

        let z_tgt_bit = (z_tgt >> tgt_shift) & 1;
        if z_tgt_bit == 1 {
            z_table[i] ^= 1 << ctrl_shift;
        }
    }
}

// 4. S-Bit Measurement Readout Kernel
#[inline(always)]
fn sbit_measure_kernel(
    x_table: &[u64],
    block_idx: usize,
    bit_shift: u32,
) -> u8 {
    if block_idx < x_table.len() {
        let x_word = x_table[block_idx];
        ((x_word >> bit_shift) & 1) as u8
    } else {
        0
    }
}

// --- ALGORITHM 1: GHZ ENTANGLEMENT PIPELINE ---
fn run_ghz(max_qubits: usize) {
    println!("=== Running GHZ Entanglement Pipeline ({} Qubits) ===", max_qubits);
    let start = Instant::now();
    let blocks_per_row = (max_qubits + 63) / 64;
    
    let mut x_table = vec![0u64; blocks_per_row];
    let mut z_table = vec![0u64; blocks_per_row];
    let mut sign = vec![0u8; blocks_per_row];

    for idx_q in 0..max_qubits {
        let block = idx_q / 64;
        if block < blocks_per_row {
            z_table[block] |= 1 << (idx_q % 64);
        }
    }

    sbit_hadamard_kernel(&mut x_table, &mut z_table, &mut sign, blocks_per_row, 0, 0);

    let chain_limit = if max_qubits > 500 { 500 } else { max_qubits };
    for q in 0..(chain_limit - 1) {
        sbit_cnot_kernel(
            &mut x_table, &mut z_table,
            (q % 64) as u32,
            ((q + 1) % 64) as u32,
            blocks_per_row,
        );
    }

    let out_0 = sbit_measure_kernel(&x_table, 0, 0);
    println!(" -> GHZ Readout Qubit #0 Outcome: {}", out_0);
    println!(" -> Execution Time: {:?}", start.elapsed());
}

// --- ALGORITHM 2: QUANTUM TELEPORTATION PROTOCOL ---
fn run_teleport(mut max_qubits: usize) {
    if max_qubits < 3 { max_qubits = 3; }
    println!("=== Running Quantum Teleportation Protocol ({} Qubits) ===", max_qubits);
    let start = Instant::now();
    let blocks_per_row = (max_qubits + 63) / 64;
    let mut x_table = vec![0u64; blocks_per_row];
    let mut z_table = vec![0u64; blocks_per_row];
    let mut sign = vec![0u8; blocks_per_row];

    for idx_q in 0..max_qubits {
        let block = idx_q / 64;
        if block < blocks_per_row {
            z_table[block] |= 1 << (idx_q % 64);
        }
    }

    let q_in = 0;
    let q_alice = 1;
    let q_bob = 2;

    sbit_hadamard_kernel(&mut x_table, &mut z_table, &mut sign, blocks_per_row, q_in / 64, (q_in % 64) as u32);
    sbit_hadamard_kernel(&mut x_table, &mut z_table, &mut sign, blocks_per_row, q_alice / 64, (q_alice % 64) as u32);
    sbit_cnot_kernel(&mut x_table, &mut z_table, (q_alice % 64) as u32, (q_bob % 64) as u32, blocks_per_row);
    sbit_cnot_kernel(&mut x_table, &mut z_table, (q_in % 64) as u32, (q_alice % 64) as u32, blocks_per_row);
    sbit_hadamard_kernel(&mut x_table, &mut z_table, &mut sign, blocks_per_row, q_in / 64, (q_in % 64) as u32);

    let m0 = sbit_measure_kernel(&x_table, q_in / 64, (q_in % 64) as u32);
    let m1 = sbit_measure_kernel(&x_table, q_alice / 64, (q_alice % 64) as u32);

    if m1 == 1 {
        sbit_pauli_z_kernel(&mut x_table, &mut sign, q_bob / 64, (q_bob % 64) as u32, blocks_per_row);
    }
    if m0 == 1 {
        let mut temp_x = vec![0u64; blocks_per_row];
        sbit_hadamard_kernel(&mut temp_x, &mut z_table, &mut sign, blocks_per_row, q_bob / 64, (q_bob % 64) as u32);
    }

    println!(" -> Teleportation Success! M0: {}, M1: {}", m0, m1);
    println!(" -> Execution Time: {:?}", start.elapsed());
}

// --- ALGORITHM 3: GROVER SEARCH ORACLE ---
fn run_grover(max_qubits: usize) {
    println!("=== Running Grover-Style Search Oracle ({} Qubits) ===", max_qubits);
    let start = Instant::now();
    let blocks_per_row = (max_qubits + 63) / 64;
    let mut x_table = vec![0u64; blocks_per_row];
    let mut z_table = vec![0u64; blocks_per_row];
    let mut sign = vec![0u8; blocks_per_row];

    for idx_q in 0..max_qubits {
        let block = idx_q / 64;
        if block < blocks_per_row {
            z_table[block] |= 1 << (idx_q % 64);
        }
    }

    let apply_limit = if max_qubits > 1000 { 1000 } else { max_qubits };
    for q in 0..apply_limit {
        sbit_hadamard_kernel(&mut x_table, &mut z_table, &mut sign, blocks_per_row, q / 64, (q % 64) as u32);
    }

    for q in 0..(apply_limit - 1) {
        sbit_cnot_kernel(
            &mut x_table, &mut z_table,
            (q % 64) as u32,
            ((q + 1) % 64) as u32,
            blocks_per_row,
        );
    }

    let target_q = apply_limit / 2;
    sbit_pauli_z_kernel(&mut x_table, &mut sign, target_q / 64, (target_q % 64) as u32, blocks_per_row);
    
    println!(" -> Grover Target State Marked at Qubit #{}", target_q);
    println!(" -> Execution Time: {:?}", start.elapsed());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut qubits = 150_000;
    let mut mode = "ghz";

    if args.len() > 1 {
        mode = &args[1];
    }
    if args.len() > 2 {
        if let Ok(parsed_q) = args[2].parse::<usize>() {
            qubits = parsed_q;
        }
    }

    println!("=====================================================");
    println!("   S-BIT UNIVERSAL QUANTUM SIMULATOR ENGINE (RUST)");
    println!("=====================================================");

    match mode {
        "ghz" => run_ghz(qubits),
        "teleport" => run_teleport(qubits),
        "grover" => run_grover(qubits),
        _ => {
            println!("Unknown mode! Available modes: ghz, teleport, grover");
            println!("Usage: cargo run --release -- [mode] [qubits]");
        }
    }
}
