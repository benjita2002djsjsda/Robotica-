/// Functions for evaluating the robustness of an MDP policy under different transition noise models.
use crate::mdp_model::value_iteration;
use std::collections::HashMap;

/// Defines alternative transition noise models for robustness evaluation.
/// Each tuple represents `(prob_left_deviation, prob_intended_direction, prob_right_deviation)`.
/// For example, (0.1, 0.8, 0.1) means 10% chance of going left, 80% center, 10% right of intended.
const ARR_TPL_F64X3_MODELOS_RUIDO: &[(f64, f64, f64)] = &[
    (0.1, 0.8, 0.1),   // 80% success
    (0.05, 0.9, 0.05), // 90% success
    (0.15, 0.7, 0.15), // 70% success
    (0.25, 0.5, 0.25), // 50% success
];

/// Constructs a transition probability map for a given noise model.
///
/// # Arguments
///
/// * `f64IzqProb` - Probability of deviating to the "left" of the intended action.
/// * `f64CentroProb` - Probability of successfully performing the intended action.
/// * `f64DerProb` - Probability of deviating to the "right" of the intended action.
///
/// "Left" and "Right" are relative to the action:
/// - For "N": Left is "O" (West), Right is "E" (East).
/// - For "S": Left is "O" (West), Right is "E" (East).
/// - For "E": Left is "N" (North), Right is "S" (South).
/// - For "O": Left is "N" (North), Right is "S" (South).
///
/// # Returns
///
/// A `HashMap<String, HashMap<String, f64>>` representing the transition model.
/// Outer keys are actions (e.g., "N"), inner keys are resulting directions (e.g., "N", "E", "O"),
/// and values are probabilities.
pub fn construir_modelo_ruido(
    f64IzqProb: f64,
    f64CentroProb: f64,
    f64DerProb: f64,
) -> HashMap<String, HashMap<String, f64>> {
    let vec_sAcciones = vec!["N", "S", "E", "O"];
    let mut hm_s_hm_s_f64Modelo = HashMap::new();

    for sAccion in &vec_sAcciones {
        let mut hm_s_f64Transiciones = HashMap::new();
        match *sAccion {
            "N" => {
                // For North action: E is right, O is left
                hm_s_f64Transiciones.insert("N".to_string(), f64CentroProb);
                hm_s_f64Transiciones.insert("E".to_string(), f64DerProb);
                hm_s_f64Transiciones.insert("O".to_string(), f64IzqProb);
            }
            "S" => {
                // For South action: E is right, O is left
                hm_s_f64Transiciones.insert("S".to_string(), f64CentroProb);
                hm_s_f64Transiciones.insert("E".to_string(), f64DerProb);
                hm_s_f64Transiciones.insert("O".to_string(), f64IzqProb);
            }
            "E" => {
                // For East action: N is left, S is right
                hm_s_f64Transiciones.insert("E".to_string(), f64CentroProb);
                hm_s_f64Transiciones.insert("N".to_string(), f64IzqProb);
                hm_s_f64Transiciones.insert("S".to_string(), f64DerProb);
            }
            "O" => {
                // For West action: N is left, S is right
                hm_s_f64Transiciones.insert("O".to_string(), f64CentroProb);
                hm_s_f64Transiciones.insert("N".to_string(), f64IzqProb);
                hm_s_f64Transiciones.insert("S".to_string(), f64DerProb);
            }
            _ => {} // Should not happen with predefined actions
        }
        hm_s_hm_s_f64Modelo.insert(sAccion.to_string(), hm_s_f64Transiciones);
    }

    hm_s_hm_s_f64Modelo
}

/// Evaluates the robustness of a given base policy by comparing it against policies
/// generated under various noise models.
///
/// Robustness is measured by the number of states in which the optimal action changes
/// when the transition probabilities are altered.
///
/// # Arguments
///
/// * `ref_hm_s_sPoliticaBase` - A reference to the base optimal policy (State -> Action).
/// * `f64Lambda` - The discount factor used for `value_iteration`.
///
/// # Returns
///
/// A `Vec<(String, usize)>` where each tuple contains:
///   - A string label for the noise model (e.g., "80%").
///   - The number of states where the policy differed from the base policy under this noise model.
pub fn evaluar_robustez(
    ref_hm_s_sPoliticaBase: &HashMap<String, String>,
    f64Lambda: f64,
) -> Vec<(String, usize)> {
    let mut vec_tpl_s_uiResultados = Vec::new();

    for (f64Izq, f64Centro, f64Der) in ARR_TPL_F64X3_MODELOS_RUIDO.iter() {
        let sEtiqueta = format!("{}%", (*f64Centro * 100.0) as usize);
        let hm_s_hm_s_f64ModeloRuido = construir_modelo_ruido(*f64Izq, *f64Centro, *f64Der);

        // Epsilon (convergence threshold) for value_iteration, can be a small constant.
        // The policy is the second element of the tuple returned by value_iteration.
        let (_, hm_s_sPoliticaAdaptada) =
            value_iteration(f64Lambda, 0.01, Some(&hm_s_hm_s_f64ModeloRuido));

        let uiCambios = ref_hm_s_sPoliticaBase
            .iter()
            .filter(|(sStateKey, sOriginalActionValue)| {
                // sStateKey: &&String, sOriginalActionValue: &&String
                match hm_s_sPoliticaAdaptada.get(*sStateKey) {
                    // get expects &String or String, so dereference sStateKey twice
                    Some(sAdaptedActionValue) => sAdaptedActionValue != *sOriginalActionValue, // sAdaptedActionValue: &String
                    None => true, // State not in adapted policy, consider it a change
                }
            })
            .count();

        println!("Ruido {}: {} cambios", sEtiqueta, uiCambios);
        vec_tpl_s_uiResultados.push((sEtiqueta, uiCambios));
    }

    vec_tpl_s_uiResultados
}
