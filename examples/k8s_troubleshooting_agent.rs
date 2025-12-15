// Self-Learning K8s Troubleshooting Agent with RuVector RAG
//
// This example demonstrates a production-ready Kubernetes troubleshooting agent
// that uses RuVector's self-learning RAG capabilities to:
// 1. Build knowledge base from K8s logs, metrics, and documentation
// 2. Retrieve relevant troubleshooting guides based on error patterns
// 3. Learn from successful resolutions to improve future recommendations
// 4. Use graph relationships to identify related issues

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use aof_memory::{
    backends::ruvector::{RuVectorBackend, RuVectorConfig, DeploymentMode},
    rag::self_learning::{SelfLearningRAG, RAGConfig, RetrievalStrategy, AgentAction},
};

/// K8s troubleshooting agent
pub struct K8sTroubleshootingAgent {
    rag: Arc<SelfLearningRAG>,
    knowledge_base: Arc<K8sKnowledgeBase>,
}

/// K8s knowledge base
pub struct K8sKnowledgeBase {
    backend: Arc<RuVectorBackend>,
}

/// K8s issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sIssue {
    pub id: String,
    pub kind: K8sResourceKind,
    pub namespace: String,
    pub name: String,
    pub error_message: String,
    pub logs: Vec<String>,
    pub events: Vec<K8sEvent>,
    pub metrics: Option<K8sMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum K8sResourceKind {
    Pod,
    Deployment,
    Service,
    Node,
    PersistentVolumeClaim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sEvent {
    pub timestamp: String,
    pub reason: String,
    pub message: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub restart_count: u32,
}

/// Troubleshooting solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingSolution {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<String>,
    pub related_issues: Vec<String>,
    pub success_rate: f32,
    pub confidence: f32,
}

impl K8sTroubleshootingAgent {
    /// Create a new K8s troubleshooting agent
    pub async fn new() -> Result<Self> {
        info!("Initializing K8s troubleshooting agent");

        // Configure RuVector backend
        let backend_config = RuVectorConfig {
            mode: DeploymentMode::Cluster,
            vector: ruvector_core::config::VectorConfig {
                dimensions: 1536,
                metric: ruvector_core::config::Metric::Cosine,
                normalize: true,
            },
            hnsw: ruvector_core::config::HNSWConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 100,
                max_layers: 6,
            },
            gnn: super::GNNConfig {
                enabled: true,
                model: "SONA-Adaptive".to_string(),
                learning_rate: 0.001,
                adaptation_interval: 100,
                feedback_weight: 0.7,
                patterns: super::PatternConfig {
                    track_success_rate: true,
                    track_query_latency: true,
                    track_retrieval_relevance: true,
                    min_patterns_for_adaptation: 50,
                },
                auto_optimize: super::AutoOptimizeConfig {
                    enabled: true,
                    schedule: "0 2 * * *".to_string(),
                    min_queries_for_optimization: 1000,
                },
            },
            // ... rest of config
        };

        let backend = Arc::new(
            RuVectorBackend::new(backend_config)
                .await
                .context("Failed to create RuVector backend")?
        );

        // Configure RAG
        let rag_config = RAGConfig {
            retrieval: super::RetrievalConfig {
                strategy: RetrievalStrategy::Hybrid,
                top_k: 10,
                reranking_enabled: true,
                hybrid: super::HybridConfig {
                    vector_weight: 0.6,
                    graph_weight: 0.4,
                },
                multi_hop: super::MultiHopConfig {
                    enabled: true,
                    max_hops: 2,
                    min_relevance_score: 0.7,
                },
            },
            learning: super::LearningConfig {
                enabled: true,
                feedback: super::FeedbackConfig {
                    collect_user_feedback: true,
                    implicit_feedback: true,
                    feedback_store: "memory://k8s-feedback".to_string(),
                },
                query_expansion: super::QueryExpansionConfig {
                    enabled: true,
                    max_expansions: 3,
                    confidence_threshold: 0.7,
                },
                ranking: super::RankingConfig {
                    strategy: super::RankingStrategy::Learned,
                    features: vec![
                        "vectorSimilarity".to_string(),
                        "graphCentrality".to_string(),
                        "successRate".to_string(),
                        "recency".to_string(),
                    ],
                },
            },
            chunking: super::ChunkingConfig {
                strategy: super::ChunkingStrategy::Semantic,
                chunk_size: 512,
                chunk_overlap: 50,
            },
        };

        let rag = Arc::new(
            SelfLearningRAG::new(backend.clone(), rag_config)
                .await
                .context("Failed to create RAG pipeline")?
        );

        let knowledge_base = Arc::new(K8sKnowledgeBase { backend });

        Ok(Self {
            rag,
            knowledge_base,
        })
    }

    /// Initialize knowledge base with K8s troubleshooting guides
    pub async fn initialize_knowledge_base(&self) -> Result<()> {
        info!("Initializing K8s knowledge base");

        // Load common K8s issues and solutions
        let common_issues = vec![
            (
                "pod-crashloopbackoff",
                "Pod CrashLoopBackOff: Container repeatedly crashing",
                vec![
                    "Check pod logs: kubectl logs <pod-name>",
                    "Inspect container exit code and reason",
                    "Verify resource limits (CPU/memory)",
                    "Check liveness/readiness probes configuration",
                    "Review application configuration and dependencies",
                ],
                vec!["pod-oomkilled", "pod-image-pull-error"],
            ),
            (
                "pod-oomkilled",
                "Pod OOMKilled: Container killed due to out of memory",
                vec![
                    "Check memory usage: kubectl top pod <pod-name>",
                    "Increase memory limits in deployment spec",
                    "Optimize application memory usage",
                    "Enable memory profiling to find leaks",
                    "Consider horizontal pod autoscaling",
                ],
                vec!["pod-crashloopbackoff", "node-memory-pressure"],
            ),
            (
                "pod-image-pull-error",
                "Pod ImagePullBackOff: Cannot pull container image",
                vec![
                    "Verify image name and tag are correct",
                    "Check image registry credentials",
                    "Ensure network connectivity to registry",
                    "Verify imagePullSecrets are configured",
                    "Check if image exists in registry",
                ],
                vec!["pod-crashloopbackoff"],
            ),
            (
                "service-no-endpoints",
                "Service has no endpoints: No pods matching selector",
                vec![
                    "Verify pod labels match service selector",
                    "Check if pods are running: kubectl get pods -l <selector>",
                    "Ensure pods are ready (readiness probe passing)",
                    "Review service and deployment YAML",
                    "Check namespace matches between service and pods",
                ],
                vec!["pod-not-ready"],
            ),
            (
                "node-not-ready",
                "Node NotReady: Node is not accepting workloads",
                vec![
                    "Check node conditions: kubectl describe node <node-name>",
                    "Verify kubelet is running on the node",
                    "Check node resource pressure (CPU/memory/disk)",
                    "Review kubelet logs for errors",
                    "Verify network connectivity to control plane",
                ],
                vec!["pod-pending", "node-memory-pressure", "node-disk-pressure"],
            ),
            (
                "pvc-pending",
                "PersistentVolumeClaim Pending: Cannot bind to storage",
                vec![
                    "Check if suitable PV exists: kubectl get pv",
                    "Verify storage class is available",
                    "Review PVC access modes and capacity",
                    "Check dynamic provisioner logs",
                    "Ensure storage backend has capacity",
                ],
                vec!["pod-pending"],
            ),
        ];

        for (id, title, steps, related) in common_issues {
            let solution = TroubleshootingSolution {
                id: id.to_string(),
                title: title.to_string(),
                description: title.to_string(),
                steps,
                related_issues: related.iter().map(|s| s.to_string()).collect(),
                success_rate: 0.0, // Will be learned
                confidence: 1.0,
            };

            self.knowledge_base.add_solution(&solution).await?;
        }

        info!("Knowledge base initialized with {} solutions", common_issues.len());
        Ok(())
    }

    /// Diagnose K8s issue
    pub async fn diagnose(&self, issue: &K8sIssue) -> Result<Vec<TroubleshootingSolution>> {
        info!("Diagnosing K8s issue: {:?} in {}/{}", issue.kind, issue.namespace, issue.name);

        // Build query from issue details
        let query = self.build_query(issue);

        // Generate embedding (in production, use actual embedding model)
        let query_vector = self.generate_embedding(&query).await?;

        // Retrieve relevant solutions using self-learning RAG
        let results = self.rag.retrieve(&query, query_vector, None).await?;

        // Convert to solutions
        let solutions: Vec<TroubleshootingSolution> = results
            .iter()
            .filter_map(|r| {
                serde_json::from_value(r.metadata.clone()).ok()
            })
            .collect();

        info!("Found {} potential solutions", solutions.len());

        Ok(solutions)
    }

    /// Build query string from issue
    fn build_query(&self, issue: &K8sIssue) -> String {
        let mut query_parts = vec![
            format!("{:?}", issue.kind),
            issue.error_message.clone(),
        ];

        // Add relevant log snippets
        for log in issue.logs.iter().take(3) {
            query_parts.push(log.clone());
        }

        // Add event messages
        for event in issue.events.iter().take(3) {
            query_parts.push(format!("{}: {}", event.reason, event.message));
        }

        query_parts.join(" | ")
    }

    /// Generate embedding for query
    async fn generate_embedding(&self, _query: &str) -> Result<ruvector_core::Vector> {
        // In production, use actual embedding model (OpenAI, HuggingFace, etc.)
        Ok(vec![0.0; 1536])
    }

    /// Apply solution and record feedback
    pub async fn apply_solution(
        &self,
        issue: &K8sIssue,
        solution: &TroubleshootingSolution,
        query_id: &str,
    ) -> Result<bool> {
        info!("Applying solution: {}", solution.title);

        // Execute solution steps (simplified for example)
        let success = self.execute_solution_steps(issue, solution).await?;

        // Record feedback based on success
        let action = if success {
            AgentAction::Used
        } else {
            AgentAction::Rejected
        };

        self.rag.record_implicit_feedback(query_id, &solution.id, action).await?;

        if success {
            info!("Solution applied successfully");
        } else {
            warn!("Solution did not resolve the issue");
        }

        Ok(success)
    }

    /// Execute solution steps
    async fn execute_solution_steps(
        &self,
        issue: &K8sIssue,
        solution: &TroubleshootingSolution,
    ) -> Result<bool> {
        // In production, execute actual kubectl commands or K8s API calls
        // For example, parse steps and execute automation

        for (i, step) in solution.steps.iter().enumerate() {
            info!("Step {}: {}", i + 1, step);

            // Parse and execute command
            if step.contains("kubectl") {
                // Execute kubectl command
                info!("Would execute: {}", step);
            }
        }

        // Check if issue is resolved
        let resolved = self.verify_resolution(issue).await?;

        Ok(resolved)
    }

    /// Verify if issue is resolved
    async fn verify_resolution(&self, issue: &K8sIssue) -> Result<bool> {
        // In production, check actual K8s resource status
        info!("Verifying resolution for {}/{}", issue.namespace, issue.name);

        // For example:
        // - Check pod status is Running
        // - Verify no error events in last 5 minutes
        // - Check metrics are stable

        Ok(true) // Simplified for example
    }

    /// Learn from resolution
    pub async fn learn_from_resolution(
        &self,
        issue: &K8sIssue,
        solution: &TroubleshootingSolution,
        success: bool,
    ) -> Result<()> {
        info!(
            "Learning from resolution: {} -> {}",
            if success { "SUCCESS" } else { "FAILURE" },
            solution.title
        );

        // Update solution success rate
        let mut updated_solution = solution.clone();
        updated_solution.success_rate = if success {
            (solution.success_rate * 0.9) + 0.1
        } else {
            solution.success_rate * 0.9
        };

        // Store updated solution
        self.knowledge_base.update_solution(&updated_solution).await?;

        // Create graph relationships for similar issues
        if success {
            self.knowledge_base.create_success_relationship(
                &issue.id,
                &solution.id,
            ).await?;
        }

        Ok(())
    }

    /// Get learning metrics
    pub async fn get_metrics(&self) -> Result<AgentMetrics> {
        let learning = self.knowledge_base.backend.get_learning_metrics().await?;
        let performance = self.knowledge_base.backend.get_metrics().await?;

        Ok(AgentMetrics {
            total_diagnoses: learning.total_queries,
            successful_resolutions: (learning.total_queries as f32 * learning.accuracy) as usize,
            knowledge_base_size: performance.vector_count,
            average_query_latency: performance.query_latency_p50,
            learning_accuracy: learning.accuracy,
        })
    }
}

impl K8sKnowledgeBase {
    /// Add solution to knowledge base
    async fn add_solution(&self, solution: &TroubleshootingSolution) -> Result<()> {
        // Generate embedding for solution
        let embedding = self.generate_solution_embedding(solution).await?;

        // Store in vector database
        self.backend.insert_with_relationships(
            &solution.id,
            embedding,
            serde_json::to_value(solution)?,
        ).await?;

        Ok(())
    }

    /// Update solution in knowledge base
    async fn update_solution(&self, solution: &TroubleshootingSolution) -> Result<()> {
        // Delete old version
        self.backend.delete(&solution.id).await?;

        // Add updated version
        self.add_solution(solution).await?;

        Ok(())
    }

    /// Create success relationship between issue and solution
    async fn create_success_relationship(
        &self,
        issue_id: &str,
        solution_id: &str,
    ) -> Result<()> {
        // Use Cypher to create relationship
        let query = r#"
            MATCH (i:Issue {id: $issue_id})
            MATCH (s:Solution {id: $solution_id})
            MERGE (i)-[r:RESOLVED_BY]->(s)
            ON CREATE SET r.count = 1
            ON MATCH SET r.count = r.count + 1
        "#;

        let params = serde_json::json!({
            "issue_id": issue_id,
            "solution_id": solution_id,
        });

        self.backend.cypher_search(query, params).await?;

        Ok(())
    }

    /// Generate embedding for solution
    async fn generate_solution_embedding(
        &self,
        solution: &TroubleshootingSolution,
    ) -> Result<ruvector_core::Vector> {
        // In production, use actual embedding model
        let text = format!(
            "{} {} {}",
            solution.title,
            solution.description,
            solution.steps.join(" "),
        );

        // Call embedding API
        Ok(vec![0.0; 1536])
    }
}

/// Agent metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_diagnoses: usize,
    pub successful_resolutions: usize,
    pub knowledge_base_size: usize,
    pub average_query_latency: std::time::Duration,
    pub learning_accuracy: f32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create agent
    let agent = K8sTroubleshootingAgent::new().await?;

    // Initialize knowledge base
    agent.initialize_knowledge_base().await?;

    // Example: Diagnose a CrashLoopBackOff issue
    let issue = K8sIssue {
        id: "issue-001".to_string(),
        kind: K8sResourceKind::Pod,
        namespace: "default".to_string(),
        name: "my-app-7d9f8c6b5-abc12".to_string(),
        error_message: "Back-off restarting failed container".to_string(),
        logs: vec![
            "Error: ECONNREFUSED connect ECONNREFUSED 127.0.0.1:5432".to_string(),
            "Failed to connect to database".to_string(),
        ],
        events: vec![
            K8sEvent {
                timestamp: "2025-12-09T10:30:00Z".to_string(),
                reason: "BackOff".to_string(),
                message: "Back-off restarting failed container".to_string(),
                event_type: "Warning".to_string(),
            },
        ],
        metrics: Some(K8sMetrics {
            cpu_usage: 0.5,
            memory_usage: 256.0,
            restart_count: 15,
        }),
    };

    // Diagnose
    let solutions = agent.diagnose(&issue).await?;

    println!("\n=== Diagnosis Results ===");
    for (i, solution) in solutions.iter().enumerate() {
        println!("\nSolution {}: {}", i + 1, solution.title);
        println!("Confidence: {:.2}%", solution.confidence * 100.0);
        println!("Success Rate: {:.2}%", solution.success_rate * 100.0);
        println!("\nSteps:");
        for (j, step) in solution.steps.iter().enumerate() {
            println!("  {}. {}", j + 1, step);
        }
    }

    // Apply first solution
    if let Some(solution) = solutions.first() {
        let query_id = "query-001";
        let success = agent.apply_solution(&issue, solution, query_id).await?;

        // Learn from outcome
        agent.learn_from_resolution(&issue, solution, success).await?;
    }

    // Get metrics
    let metrics = agent.get_metrics().await?;
    println!("\n=== Agent Metrics ===");
    println!("Total Diagnoses: {}", metrics.total_diagnoses);
    println!("Successful Resolutions: {}", metrics.successful_resolutions);
    println!("Knowledge Base Size: {}", metrics.knowledge_base_size);
    println!("Average Query Latency: {:?}", metrics.average_query_latency);
    println!("Learning Accuracy: {:.2}%", metrics.learning_accuracy * 100.0);

    Ok(())
}
