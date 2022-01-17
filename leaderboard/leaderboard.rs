use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use linked_hash_map::LinkedHashMap;

type Level = u32;
type Points = u64;
type PlayerId = u64;
type ScoreId = u64;

#[derive(Debug)]
struct Leaderboard {
    board: HashMap<Level, BTreeMap<Reverse<Points>, LinkedHashMap<ScoreId, Score>>>
}

#[derive(Debug, Copy, Clone)]
struct Score {
    score_id: ScoreId,
    player_id: PlayerId,
    level: Level,
    points: Points
}

impl Leaderboard {
    fn new() -> Self {
        Leaderboard { board: HashMap::new() }
    }

    fn get_top_scores(&self, level: Level, n: usize) -> Vec<&Score> {
        self.board.get(&level)
            .map(|scores_at_level|
                scores_at_level.values()
                    .flat_map(|scores| scores.values())
                    .take(n)
                    .collect::<Vec<&Score>>())
            .unwrap_or(vec![])
    }

    fn update_score(&mut self, score: Score) {
        self.board.entry(score.level)
            .or_insert(BTreeMap::new())
            .entry(Reverse(score.points))
            .or_insert(LinkedHashMap::new())
            .insert(score.score_id, score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scores_one_level_one_player() {
        let mut leaderboard = Leaderboard::new();
        leaderboard.update_score(Score { score_id: 1, player_id: 1, level: 1, points: 100 });
        leaderboard.update_score(Score { score_id: 2, player_id: 1, level: 1, points: 200 });
        leaderboard.update_score(Score { score_id: 3, player_id: 1, level: 1, points: 50 });
        leaderboard.update_score(Score { score_id: 4, player_id: 1, level: 1, points: 10 });

        let top_scores = leaderboard.get_top_scores(1, 2);
        assert_eq!(top_scores.len(), 2);
        assert_eq!(top_scores[0].score_id, 2);
        assert_eq!(top_scores[0].points, 200);
        assert_eq!(top_scores[1].score_id, 1);
        assert_eq!(top_scores[1].points, 100);
    }

    #[test]
    fn test_scores_one_level_many_players() {
        let mut leaderboard = Leaderboard::new();
        leaderboard.update_score(Score { score_id: 1, player_id: 1, level: 1, points: 100 });
        leaderboard.update_score(Score { score_id: 2, player_id: 2, level: 1, points: 200 });
        leaderboard.update_score(Score { score_id: 3, player_id: 3, level: 1, points: 50 });
        leaderboard.update_score(Score { score_id: 4, player_id: 4, level: 1, points: 10 });

        let top_scores = leaderboard.get_top_scores(1, 2);
        assert_eq!(top_scores.len(), 2);
        assert_eq!(top_scores[0].score_id, 2);
        assert_eq!(top_scores[0].player_id, 2);
        assert_eq!(top_scores[0].points, 200);
        assert_eq!(top_scores[1].score_id, 1);
        assert_eq!(top_scores[1].player_id, 1);
        assert_eq!(top_scores[1].points, 100);
    }

    #[test]
    fn test_scores_many_levels_many_players() {
        let mut leaderboard = Leaderboard::new();
        leaderboard.update_score(Score { score_id: 1, player_id: 1, level: 1, points: 100 });
        leaderboard.update_score(Score { score_id: 2, player_id: 2, level: 2, points: 200 });
        leaderboard.update_score(Score { score_id: 3, player_id: 3, level: 1, points: 50 });
        leaderboard.update_score(Score { score_id: 4, player_id: 4, level: 2, points: 10 });

        let top_scores_level1 = leaderboard.get_top_scores(1, 2);
        let top_scores_level2 = leaderboard.get_top_scores(2, 2);
        assert_eq!(top_scores_level1.len(), 2);
        assert_eq!(top_scores_level1[0].score_id, 1);
        assert_eq!(top_scores_level1[0].player_id, 1);
        assert_eq!(top_scores_level1[0].points, 100);
        assert_eq!(top_scores_level1[1].score_id, 3);
        assert_eq!(top_scores_level1[1].player_id, 3);
        assert_eq!(top_scores_level1[1].points, 50);
        assert_eq!(top_scores_level2.len(), 2);
        assert_eq!(top_scores_level2[0].score_id, 2);
        assert_eq!(top_scores_level2[0].player_id, 2);
        assert_eq!(top_scores_level2[0].points, 200);
        assert_eq!(top_scores_level2[1].score_id, 4);
        assert_eq!(top_scores_level2[1].player_id, 4);
        assert_eq!(top_scores_level2[1].points, 10);
    }
}
