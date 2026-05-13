use eframe::egui;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Player {
    One,
    Two,
}

#[derive(Clone)]
struct Board {
    pockets: [u32; 14],
    current_player: Player,
}

impl Board {
    fn new() -> Self {
        let mut pockets = [4; 14];
        pockets[6] = 0;
        pockets[13] = 0;
        Board {
            pockets,
            current_player: Player::One,
        }
    }

    fn play(&mut self, pit_idx: usize) -> Result<(), &str> {
        if !self.is_own_pit(pit_idx) { return Err("自分の穴を選んでください"); }
        if self.pockets[pit_idx] == 0 { return Err("その穴は空です"); }

        let mut stones = self.pockets[pit_idx];
        self.pockets[pit_idx] = 0;
        let mut current = pit_idx;

        while stones > 0 {
            current = (current + 1) % 14;
            if (self.current_player == Player::One && current == 13) ||
               (self.current_player == Player::Two && current == 6) {
                continue;
            }
            self.pockets[current] += 1;
            stones -= 1;
        }

        if self.is_own_pit(current) && self.pockets[current] == 1 {
            let opposite = 12 - current;
            if self.pockets[opposite] > 0 {
                let captured = self.pockets[opposite] + 1;
                self.pockets[opposite] = 0;
                self.pockets[current] = 0;
                let store_idx = if self.current_player == Player::One { 6 } else { 13 };
                self.pockets[store_idx] += captured;
            }
        }

        let store_landed = (self.current_player == Player::One && current == 6) ||
                           (self.current_player == Player::Two && current == 13);
        
        if !store_landed {
            self.current_player = match self.current_player {
                Player::One => Player::Two,
                Player::Two => Player::One,
            };
        }
        Ok(())
    }

    fn is_own_pit(&self, idx: usize) -> bool {
        match self.current_player {
            Player::One => idx <= 5,
            Player::Two => (7..=12).contains(&idx),
        }
    }

    fn is_game_over(&self) -> bool {
        self.pockets[0..6].iter().all(|&c| c == 0) || self.pockets[7..13].iter().all(|&c| c == 0)
    }

    fn evaluate(&self) -> i32 {
        (self.pockets[13] as i32) - (self.pockets[6] as i32)
    }

    // アルファ・ベータ枝刈りによる高速化
    fn alphabeta(&self, depth: u32, mut alpha: i32, mut beta: i32, is_maximizing: bool) -> i32 {
        if depth == 0 || self.is_game_over() {
            return self.evaluate();
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;
            for i in 7..13 {
                if self.pockets[i] > 0 {
                    let mut temp = self.clone();
                    let _ = temp.play(i);
                    let eval = temp.alphabeta(depth - 1, alpha, beta, temp.current_player == Player::Two);
                    max_eval = max_eval.max(eval);
                    alpha = alpha.max(eval);
                    if beta <= alpha { break; }
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            for i in 0..6 {
                if self.pockets[i] > 0 {
                    let mut temp = self.clone();
                    let _ = temp.play(i);
                    let eval = temp.alphabeta(depth - 1, alpha, beta, temp.current_player == Player::Two);
                    min_eval = min_eval.min(eval);
                    beta = beta.min(eval);
                    if beta <= alpha { break; }
                }
            }
            min_eval
        }
    }

    fn get_best_move(&self, depth: u32) -> Option<usize> {
        let mut best_move = None;
        let mut max_eval = i32::MIN;
        for i in 7..13 {
            if self.pockets[i] > 0 {
                let mut temp = self.clone();
                let _ = temp.play(i);
                let eval = temp.alphabeta(depth - 1, i32::MIN, i32::MAX, temp.current_player == Player::Two);
                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some(i);
                }
            }
        }
        best_move
    }

    fn finalize_game(&mut self) {
        let p1_rem: u32 = self.pockets[0..6].iter().sum();
        let p2_rem: u32 = self.pockets[7..13].iter().sum();
        for i in 0..6 { self.pockets[i] = 0; }
        for i in 7..13 { self.pockets[i] = 0; }
        self.pockets[6] += p1_rem;
        self.pockets[13] += p2_rem;
    }

    fn winner(&self) -> Option<Player> {
        if self.pockets[6] > self.pockets[13] { Some(Player::One) }
        else if self.pockets[13] > self.pockets[6] { Some(Player::Two) }
        else { None }
    }
}

struct MancalaApp {
    board: Board,
    message: String,
    ai_depth: u32,
}

impl MancalaApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            board: Board::new(),
            message: "Your Turn".to_string(),
            ai_depth: 5,
        }
    }
}

// 共通の描画ヘルパー
fn draw_pocket(ui: &mut egui::Ui, count: u32, enabled: bool, size: egui::Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    
    let fill_color = if enabled { 
        ui.visuals().widgets.active.bg_fill 
    } else { 
        ui.visuals().widgets.noninteractive.bg_fill 
    };
    
    ui.painter().rect_filled(rect, 8.0, fill_color);
    ui.painter().rect_stroke(rect, 8.0, visuals.fg_stroke);

    let stone_radius = if count > 15 { 3.0 } else { 4.5 };
    let stones_per_row = (size.x / 14.0) as u32;

    for i in 0..count {
        let row = i / stones_per_row;
        let col = i % stones_per_row;
        let center = rect.min + egui::vec2(10.0 + col as f32 * 11.0, 10.0 + row as f32 * 11.0);
        
        ui.painter().circle_filled(center, stone_radius, egui::Color32::from_rgb(180, 190, 255));
        ui.painter().circle_stroke(center, stone_radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
    }
    response
}

impl eframe::App for MancalaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ロジック更新
        if self.board.is_game_over() {
            self.board.finalize_game();
            let result = match self.board.winner() {
                Some(Player::One) => "Player One Wins!",
                Some(Player::Two) => "AI Wins!",
                None => "Draw Game!",
            };
            self.message = format!("Game Over: {}", result);
        } else if self.board.current_player == Player::Two {
            if let Some(best_idx) = self.board.get_best_move(self.ai_depth) {
                let _ = self.board.play(best_idx);
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rust Mancala: Human vs AI");
                ui.label(egui::RichText::new(&self.message).size(18.0).strong());
            });
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                // P2ストア
                ui.vertical(|ui| {
                    ui.label("AI Score");
                    draw_pocket(ui, self.board.pockets[13], false, egui::vec2(60.0, 150.0));
                });

                ui.vertical(|ui| {
                    // P2ポケット
                    ui.horizontal(|ui| {
                        for i in (7..=12).rev() {
                            draw_pocket(ui, self.board.pockets[i], false, egui::vec2(60.0, 70.0));
                        }
                    });
                    ui.add_space(10.0);
                    // P1ポケット
                    ui.horizontal(|ui| {
                        for i in 0..=5 {
                            let count = self.board.pockets[i];
                            let can_play = self.board.current_player == Player::One && count > 0 && !self.board.is_game_over();
                            if draw_pocket(ui, count, can_play, egui::vec2(60.0, 70.0)).clicked() && can_play {
                                let _ = self.board.play(i);
                            }
                        }
                    });
                });

                // P1ストア
                ui.vertical(|ui| {
                    ui.label("Your Score");
                    draw_pocket(ui, self.board.pockets[6], false, egui::vec2(60.0, 150.0));
                });
            });

            ui.add_space(30.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("AI Level:");
                ui.add(egui::Slider::new(&mut self.ai_depth, 1..=10));
                if ui.button("New Game").clicked() {
                    self.board = Board::new();
                    self.message = "Your Turn".to_string();
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([650.0, 450.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Mancala Rust AI",
        options,
        Box::new(|cc| Box::new(MancalaApp::new(cc))),
    )
}
