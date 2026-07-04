use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ── 游戏数据类型 ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Choice {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RoundResult {
    Win,
    Lose,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundRecord {
    round: u32,
    player_choice: Choice,
    computer_choice: Choice,
    result: RoundResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameStats {
    total_rounds: u32,
    player_wins: u32,
    computer_wins: u32,
    draws: u32,
    win_rate: f64,
    history: Vec<RoundRecord>,
}

#[derive(Debug, Clone)]
struct GameState {
    rounds: Vec<RoundRecord>,
}

impl GameState {
    fn new() -> Self {
        Self { rounds: Vec::new() }
    }

    fn play(&mut self, player_choice: Choice) -> RoundRecord {
        // 电脑随机选择
        let mut rng = rand::thread_rng();
        let computer_choice = match rng.gen_range(0..3) {
            0 => Choice::Rock,
            1 => Choice::Paper,
            _ => Choice::Scissors,
        };

        let result = judge(&player_choice, &computer_choice);
        let round_num = (self.rounds.len() + 1) as u32;

        let record = RoundRecord {
            round: round_num,
            player_choice: player_choice.clone(),
            computer_choice,
            result: result.clone(),
        };
        self.rounds.push(record.clone());
        record
    }

    fn stats(&self) -> GameStats {
        let total = self.rounds.len() as u32;
        let player_wins = self
            .rounds
            .iter()
            .filter(|r| r.result == RoundResult::Win)
            .count() as u32;
        let computer_wins = self
            .rounds
            .iter()
            .filter(|r| r.result == RoundResult::Lose)
            .count() as u32;
        let draws = self
            .rounds
            .iter()
            .filter(|r| r.result == RoundResult::Draw)
            .count() as u32;

        let win_rate = if total > 0 {
            player_wins as f64 / total as f64 * 100.0
        } else {
            0.0
        };

        // 返回最近 20 条历史记录
        let history: Vec<RoundRecord> = self.rounds.iter().rev().take(20).cloned().collect();

        GameStats {
            total_rounds: total,
            player_wins,
            computer_wins,
            draws,
            win_rate: (win_rate * 100.0).round() / 100.0,
            history,
        }
    }

    fn reset(&mut self) {
        self.rounds.clear();
    }
}

fn judge(player: &Choice, computer: &Choice) -> RoundResult {
    use Choice::*;
    match (player, computer) {
        (Rock, Scissors) | (Scissors, Paper) | (Paper, Rock) => RoundResult::Win,
        (Rock, Paper) | (Scissors, Rock) | (Paper, Scissors) => RoundResult::Lose,
        _ => RoundResult::Draw,
    }
}

// ── 请求/响应类型 ──

#[derive(Debug, Deserialize)]
struct PlayRequest {
    choice: Choice,
}

#[derive(Debug, Serialize)]
struct PlayResponse {
    player_choice: Choice,
    computer_choice: Choice,
    result: RoundResult,
    stats: GameStats,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

// ── 共享状态 ──

type AppState = Arc<Mutex<GameState>>;

// ── 路由处理函数 ──

async fn play(
    State(state): State<AppState>,
    Json(req): Json<PlayRequest>,
) -> Result<Json<PlayResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut game = state.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "服务器内部错误".to_string(),
            }),
        )
    })?;

    let record = game.play(req.choice);
    let stats = game.stats();

    Ok(Json(PlayResponse {
        player_choice: record.player_choice,
        computer_choice: record.computer_choice,
        result: record.result,
        stats,
    }))
}

async fn get_stats(State(state): State<AppState>) -> Json<GameStats> {
    let game = state.lock().unwrap();
    Json(game.stats())
}

async fn reset(State(state): State<AppState>) -> Json<GameStats> {
    let mut game = state.lock().unwrap();
    game.reset();
    Json(game.stats())
}

async fn index() -> Html<&'static str> {
    Html(HTML_PAGE)
}

// ── 前端 HTML 页面 ──

const HTML_PAGE: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>石头剪刀布 - 多轮对战</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            color: #e0e0e0;
        }
        .container {
            background: rgba(255,255,255,0.06);
            backdrop-filter: blur(20px);
            border-radius: 24px;
            padding: 40px 36px;
            max-width: 600px;
            width: 95%;
            box-shadow: 0 20px 60px rgba(0,0,0,0.4);
            border: 1px solid rgba(255,255,255,0.1);
        }
        h1 {
            text-align: center;
            font-size: 2rem;
            margin-bottom: 8px;
            background: linear-gradient(90deg, #f7971e, #ffd200);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        .subtitle {
            text-align: center;
            color: #888;
            margin-bottom: 28px;
            font-size: 0.9rem;
        }

        /* 统计面板 */
        .stats-panel {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 12px;
            margin-bottom: 28px;
        }
        .stat-card {
            background: rgba(255,255,255,0.08);
            border-radius: 14px;
            padding: 16px 10px;
            text-align: center;
            border: 1px solid rgba(255,255,255,0.08);
        }
        .stat-value {
            font-size: 1.8rem;
            font-weight: 700;
            color: #ffd200;
        }
        .stat-label {
            font-size: 0.75rem;
            color: #888;
            margin-top: 4px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        /* 对战区域 */
        .battle-area {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 20px;
            margin-bottom: 24px;
            min-height: 120px;
        }
        .choice-display {
            text-align: center;
            transition: all 0.3s ease;
        }
        .choice-icon {
            font-size: 4rem;
            background: rgba(255,255,255,0.08);
            width: 90px;
            height: 90px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            border: 3px solid transparent;
            transition: all 0.3s ease;
        }
        .choice-display.winner .choice-icon {
            border-color: #4caf50;
            background: rgba(76,175,80,0.2);
            box-shadow: 0 0 30px rgba(76,175,80,0.4);
            animation: pulse 0.6s ease-in-out;
        }
        .choice-display.loser .choice-icon {
            border-color: #f44336;
            background: rgba(244,67,54,0.2);
        }
        .choice-label {
            margin-top: 8px;
            font-size: 0.85rem;
            color: #aaa;
        }
        .vs-text {
            font-size: 1.5rem;
            font-weight: 700;
            color: #ff4444;
        }

        @keyframes pulse {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.15); }
        }

        /* 结果提示 */
        .result-banner {
            text-align: center;
            font-size: 1.4rem;
            font-weight: 700;
            margin-bottom: 24px;
            padding: 12px;
            border-radius: 12px;
            transition: all 0.3s ease;
        }
        .result-banner.win {
            background: rgba(76,175,80,0.2);
            color: #66bb6a;
        }
        .result-banner.lose {
            background: rgba(244,67,54,0.2);
            color: #ef5350;
        }
        .result-banner.draw {
            background: rgba(255,152,0,0.2);
            color: #ffa726;
        }

        /* 选择按钮 */
        .choices {
            display: flex;
            justify-content: center;
            gap: 16px;
            margin-bottom: 20px;
        }
        .choice-btn {
            width: 80px;
            height: 80px;
            border-radius: 50%;
            border: 2px solid rgba(255,255,255,0.2);
            background: rgba(255,255,255,0.06);
            font-size: 2.5rem;
            cursor: pointer;
            transition: all 0.25s ease;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .choice-btn:hover {
            transform: translateY(-4px);
            border-color: #ffd200;
            background: rgba(255,210,0,0.15);
            box-shadow: 0 8px 25px rgba(255,210,0,0.2);
        }
        .choice-btn:active {
            transform: scale(0.93);
        }
        .choice-btn:disabled {
            opacity: 0.4;
            cursor: not-allowed;
            transform: none;
        }

        .reset-btn {
            display: block;
            margin: 0 auto;
            padding: 10px 28px;
            border-radius: 25px;
            border: 1px solid rgba(255,255,255,0.25);
            background: transparent;
            color: #ccc;
            font-size: 0.9rem;
            cursor: pointer;
            transition: all 0.2s ease;
        }
        .reset-btn:hover {
            background: rgba(255,255,255,0.1);
            border-color: #ff5252;
            color: #ff5252;
        }

        /* 历史记录 */
        .history-section {
            margin-top: 24px;
            max-height: 200px;
            overflow-y: auto;
        }
        .history-section h3 {
            font-size: 0.9rem;
            color: #888;
            margin-bottom: 10px;
            letter-spacing: 1px;
            text-transform: uppercase;
        }
        .history-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 8px 12px;
            border-radius: 8px;
            margin-bottom: 4px;
            background: rgba(255,255,255,0.04);
            font-size: 0.85rem;
        }
        .history-item .round-num {
            color: #666;
            width: 40px;
        }
        .history-item .battle-icons {
            display: flex;
            align-items: center;
            gap: 8px;
            flex: 1;
        }
        .history-badge {
            padding: 2px 10px;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
        }
        .history-badge.win { background: rgba(76,175,80,0.2); color: #66bb6a; }
        .history-badge.lose { background: rgba(244,67,54,0.2); color: #ef5350; }
        .history-badge.draw { background: rgba(255,152,0,0.2); color: #ffa726; }

        /* 滚动条美化 */
        .history-section::-webkit-scrollbar { width: 5px; }
        .history-section::-webkit-scrollbar-track { background: transparent; }
        .history-section::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 10px; }

        .loading-dots {
            display: inline-block;
            animation: dot-flash 1s infinite;
        }
        @keyframes dot-flash {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.3; }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>✊ ✋ ✌️</h1>
        <p class="subtitle">石头剪刀布 · 多轮对战</p>

        <!-- 统计面板 -->
        <div class="stats-panel">
            <div class="stat-card">
                <div class="stat-value" id="totalRounds">0</div>
                <div class="stat-label">总轮数</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="playerWins">0</div>
                <div class="stat-label">胜利</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="computerWins">0</div>
                <div class="stat-label">失败</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="winRate">0%</div>
                <div class="stat-label">胜率</div>
            </div>
        </div>

        <!-- 对战区域 -->
        <div class="battle-area" id="battleArea">
            <div class="choice-display" id="playerDisplay">
                <div class="choice-icon">❓</div>
                <div class="choice-label">你</div>
            </div>
            <div class="vs-text">VS</div>
            <div class="choice-display" id="computerDisplay">
                <div class="choice-icon">❓</div>
                <div class="choice-label">电脑</div>
            </div>
        </div>

        <!-- 结果横幅 -->
        <div class="result-banner" id="resultBanner" style="visibility: hidden;">
            等待开局...
        </div>

        <!-- 选择按钮 -->
        <div class="choices">
            <button class="choice-btn" data-choice="rock" title="石头">✊</button>
            <button class="choice-btn" data-choice="paper" title="布">✋</button>
            <button class="choice-btn" data-choice="scissors" title="剪刀">✌️</button>
        </div>

        <button class="reset-btn" id="resetBtn">🔄 重新开始</button>

        <!-- 历史记录 -->
        <div class="history-section">
            <h3>📋 对战历史</h3>
            <div id="historyList"></div>
        </div>
    </div>

    <script>
        const CHOICE_EMOJI = { rock: '✊', paper: '✋', scissors: '✌️' };
        const CHOICE_CN = { rock: '石头', paper: '布', scissors: '✌️' };

        const playerDisplay = document.getElementById('playerDisplay');
        const computerDisplay = document.getElementById('computerDisplay');
        const resultBanner = document.getElementById('resultBanner');
        const historyList = document.getElementById('historyList');
        const choiceBtns = document.querySelectorAll('.choice-btn');
        const resetBtn = document.getElementById('resetBtn');

        let isAnimating = false;

        // 取初始统计
        fetchStats();

        choiceBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                if (isAnimating) return;
                const choice = btn.dataset.choice;
                play(choice);
            });
        });

        resetBtn.addEventListener('click', () => {
            if (isAnimating) return;
            resetGame();
        });

        async function play(choice) {
            isAnimating = true;
            choiceBtns.forEach(b => b.disabled = true);

            // 显示玩家选择
            updateDisplay(playerDisplay, choice, '');
            updateDisplay(computerDisplay, 'rock', ''); // 占位
            computerDisplay.querySelector('.choice-icon').textContent = '🤔';
            resultBanner.style.visibility = 'visible';
            resultBanner.textContent = '电脑思考中...';
            resultBanner.className = 'result-banner';

            try {
                const resp = await fetch('/api/play', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ choice: choice }),
                });
                const data = await resp.json();

                // 短暂延迟让动画更自然
                setTimeout(() => {
                    updateDisplay(playerDisplay, data.player_choice, data.result === 'win' ? 'winner' : (data.result === 'lose' ? 'loser' : ''));
                    updateDisplay(computerDisplay, data.computer_choice, data.result === 'lose' ? 'winner' : (data.result === 'win' ? 'loser' : ''));

                    // 显示结果
                    const resultText = data.result === 'win' ? '🎉 你赢了！' :
                                       data.result === 'lose' ? '😢 你输了！' :
                                       '🤝 平局！';
                    resultBanner.textContent = resultText;
                    resultBanner.className = 'result-banner ' + data.result;

                    // 更新统计
                    updateStats(data.stats);
                    updateHistory(data.stats.history);

                    isAnimating = false;
                    choiceBtns.forEach(b => b.disabled = false);
                }, 500);
            } catch (e) {
                resultBanner.textContent = '网络错误，请重试';
                resultBanner.className = 'result-banner lose';
                isAnimating = false;
                choiceBtns.forEach(b => b.disabled = false);
            }
        }

        async function fetchStats() {
            try {
                const resp = await fetch('/api/stats');
                const data = await resp.json();
                updateStats(data);
                updateHistory(data.history);
            } catch (e) {
                console.error('获取统计失败:', e);
            }
        }

        async function resetGame() {
            isAnimating = true;
            try {
                const resp = await fetch('/api/reset', { method: 'POST' });
                const data = await resp.json();
                updateStats(data);
                updateHistory(data.history);

                // 重置显示
                playerDisplay.querySelector('.choice-icon').textContent = '❓';
                playerDisplay.className = 'choice-display';
                computerDisplay.querySelector('.choice-icon').textContent = '❓';
                computerDisplay.className = 'choice-display';
                resultBanner.style.visibility = 'hidden';
            } catch (e) {
                console.error('重置失败:', e);
            }
            isAnimating = false;
        }

        function updateDisplay(el, choice, cssClass) {
            el.querySelector('.choice-icon').textContent = CHOICE_EMOJI[choice] || '❓';
            el.className = 'choice-display ' + cssClass;
        }

        function updateStats(stats) {
            document.getElementById('totalRounds').textContent = stats.total_rounds;
            document.getElementById('playerWins').textContent = stats.player_wins;
            document.getElementById('computerWins').textContent = stats.computer_wins;
            document.getElementById('winRate').textContent = stats.win_rate + '%';
        }

        function updateHistory(history) {
            if (!history || history.length === 0) {
                historyList.innerHTML = '<div style="color:#555;text-align:center;padding:20px;">暂无对战记录</div>';
                return;
            }
            historyList.innerHTML = history.map(h => {
                const badgeClass = h.result;
                const badgeText = h.result === 'win' ? '胜' : h.result === 'lose' ? '负' : '平';
                const pEmoji = CHOICE_EMOJI[h.player_choice];
                const cEmoji = CHOICE_EMOJI[h.computer_choice];
                return `<div class="history-item">
                    <span class="round-num">#${h.round}</span>
                    <span class="battle-icons">${pEmoji} vs ${cEmoji}</span>
                    <span class="history-badge ${badgeClass}">${badgeText}</span>
                </div>`;
            }).join('');
        }
    </script>
</body>
</html>"#;

// ── 主函数 ──

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(GameState::new()));

    let app = Router::new()
        .route("/", get(index))
        .route("/api/play", post(play))
        .route("/api/stats", get(get_stats))
        .route("/api/reset", post(reset))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3456);

    let addr = format!("0.0.0.0:{}", port);
    println!("🎮 石头剪刀布游戏服务器启动: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}