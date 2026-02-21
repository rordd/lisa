# Καταγραφή Ελέγχου (Audit Logging) για το ZeroClaw

> ⚠️ **Κατάσταση: Πρόταση / Οδικός Χάρτης (Roadmap)**
>
> Αυτό το έγγραφο περιγράφει προτεινόμενες προσεγγίσεις και ενδέχεται να περιλαμβάνει υποθετικές εντολές ή ρυθμίσεις.
> Για την τρέχουσα συμπεριφορά, δείτε τα: config-reference.md, operations-runbook.md, και troubleshooting.md.

## Πρόβλημα
Το ZeroClaw καταγράφει ενέργειες, αλλά στερείται ιχνών ελέγχου (audit trails) με απόδειξη παραποίησης για:
- Ποιος εκτέλεσε ποια εντολή
- Πότε και από ποιο κανάλι
- Ποιοι πόροι προσπελάστηκαν
- Εάν ενεργοποιήθηκαν πολιτικές ασφαλείας

---

## Προτεινόμενη Μορφή Log Ελέγχου

{
  "timestamp": "2026-02-16T12:34:56Z",
  "event_id": "evt_1a2b3c4d",
  "event_type": "command_execution",
  "actor": {
    "channel": "telegram",
    "user_id": "123456789",
    "username": "@alice"
  },
  "action": {
    "command": "ls -la",
    "risk_level": "low",
    "approved": false,
    "allowed": true
  },
  "result": {
    "success": true,
    "exit_code": 0,
    "duration_ms": 15
  },
  "security": {
    "policy_violation": false,
    "rate_limit_remaining": 19
  },
  "signature": "SHA256:abc123..."  // HMAC για απόδειξη μη παραποίησης
}



---

## Υλοποίηση (Implementation)

// src/security/audit.rs
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub event_id: String,
    pub event_type: AuditEventType,
    pub actor: Actor,
    pub action: Action,
    pub result: ExecutionResult,
    pub security: SecurityContext,
}

pub enum AuditEventType {
    CommandExecution,
    FileAccess,
    ConfigurationChange,
    AuthSuccess,
    AuthFailure,
    PolicyViolation,
}

pub struct AuditLogger {
    log_path: PathBuf,
    signing_key: Option<hmac::Hmac<sha2::Sha256>>,
}

impl AuditLogger {
    pub fn log(&self, event: &AuditEvent) -> anyhow::Result<()> {
        let mut line = serde_json::to_string(event)?;

        // Προσθήκη υπογραφής HMAC εάν έχει ρυθμιστεί κλειδί
        if let Some(ref key) = self.signing_key {
            let signature = compute_hmac(key, line.as_bytes());
            line.push_str(&format!("\n\"signature\": \"{}\"", signature));
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", line)?;
        file.sync_all()?;  // Αναγκαστικό άδειασμα buffer για ανθεκτικότητα
        Ok(())
    }
}

---

## Σχήμα Ρυθμίσεων (Config Schema)

[security.audit]
enabled = true
log_path = "~/.config/zeroclaw/audit.log"
max_size_mb = 100
rotate = "daily"  # daily | weekly | size

# Απόδειξη παραποίησης (Tamper evidence)
sign_events = true
signing_key_path = "~/.config/zeroclaw/audit.key"

# Τι θα καταγράφεται
log_commands = true
log_file_access = true
log_auth_events = true
log_policy_violations = true

---

## CLI Ερωτημάτων Ελέγχου

# Εμφάνιση όλων των εντολών που εκτελέστηκαν από τον χρήστη @alice
zeroclaw audit --user @alice

# Εμφάνιση όλων των εντολών υψηλού κινδύνου
zeroclaw audit --risk high

# Εμφάνιση παραβιάσεων των τελευταίων 24 ωρών
zeroclaw audit --since 24h --violations-only

# Επαλήθευση ακεραιότητας των logs
zeroclaw audit --verify-signatures

---

## Εναλλαγή Αρχείων Log (Log Rotation)



pub fn rotate_audit_log(log_path: &PathBuf, max_size: u64) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(log_path)?;
    if metadata.len() < max_size {
        return Ok(());
    }

    // Εναλλαγή: audit.log -> audit.log.1 -> audit.log.2 -> ...
    // [Ο κώδικας διαχείρισης αρχείων]
    Ok(())
}

---

## Προτεραιότητα Υλοποίησης

| Φάση | Λειτουργία | Κόπος | Αξία Ασφαλείας |
|-------|---------|--------|----------------|
| **P0** | Βασική καταγραφή συμβάντων | Χαμηλός | Μέτρια |
| **P1** | CLI ερωτημάτων (Query CLI) | Μέτριος | Μέτρια |
| **P2** | Υπογραφή HMAC | Μέτριος | Υψηλή |
| **P3** | Εναλλαγή αρχείων + Αρχειοθέτηση | Χαμηλός | Μέτρια |