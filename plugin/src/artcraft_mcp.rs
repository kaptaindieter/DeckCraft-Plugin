// ArtCraft Suite — MCA / MCP Command Executor for SlideApp

impl SlideApp {
    fn ensure_active_presentation_for_agent(&mut self, theme_opt: Option<&str>) {
        if self.session.active().is_none() {
            let th = theme_opt.unwrap_or("DeckCraft");
            let _ = self.run("file.new", serde_json::json!({"theme": th}));
        }
        self.backstage = None;
        self.sync_backstage();
    }

    fn is_only_slide_blank(&self) -> bool {
        if let Some(d) = self.session.active() {
            if d.doc.slides.len() == 1 {
                return d.doc.slides[0].shapes.iter().all(|sh| sh.text.as_ref().is_none_or(|tb| tb.is_empty()));
            }
        }
        false
    }

    fn extract_bullets_from_value(v: &serde_json::Value) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(arr) = v.get("bullets").or_else(|| v.get("points")).or_else(|| v.get("items")).and_then(serde_json::Value::as_array) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    let clean = s.trim().trim_start_matches(['-', '*', '•']).trim();
                    if !clean.is_empty() {
                        out.push(clean.to_string());
                    }
                }
            }
        }
        if out.is_empty() {
            if let Some(body) = v.get("body").or_else(|| v.get("content")).or_else(|| v.get("text")).and_then(serde_json::Value::as_str) {
                for line in body.lines() {
                    let clean = line.trim().trim_start_matches(['-', '*', '•']).trim();
                    if !clean.is_empty() {
                        out.push(clean.to_string());
                    }
                }
            }
        }
        out
    }

    fn add_rich_slide_from_spec(&mut self, spec: &serde_json::Value) -> Result<serde_json::Value, String> {
        let theme_opt = spec.get("theme").and_then(serde_json::Value::as_str);
        self.ensure_active_presentation_for_agent(theme_opt);

        let raw_layout = spec.get("layout").and_then(serde_json::Value::as_str).unwrap_or("titleAndContent");
        let title = spec.get("title").or_else(|| spec.get("heading")).and_then(serde_json::Value::as_str).unwrap_or("Neue Folie");
        let subtitle = spec.get("subtitle").and_then(serde_json::Value::as_str).unwrap_or("");
        let bg_color = spec.get("background").or_else(|| spec.get("bgColor")).or_else(|| spec.get("backgroundColor")).and_then(serde_json::Value::as_str);
        let accent_color = spec.get("accentColor").or_else(|| spec.get("accent")).and_then(serde_json::Value::as_str).unwrap_or("#38BDF8");
        let bullets = Self::extract_bullets_from_value(spec);
        let cards_arr = spec.get("cards").or_else(|| spec.get("columns")).or_else(|| spec.get("metrics")).and_then(serde_json::Value::as_array).cloned().unwrap_or_default();

        let reuse_first = self.is_only_slide_blank();
        let layout_lower = raw_layout.to_lowercase();

        if layout_lower == "title" || layout_lower.contains("hero") || (reuse_first && !subtitle.is_empty() && bullets.is_empty() && cards_arr.is_empty()) {
            if !reuse_first {
                let _ = self.run("slide.new", serde_json::json!({"layout": "title", "title": title, "body": subtitle}));
            } else {
                let _ = self.run("text.set", serde_json::json!({"id": 1, "text": title}));
                if !subtitle.is_empty() {
                    let _ = self.run("text.set", serde_json::json!({"id": 2, "text": subtitle}));
                } else if !bullets.is_empty() {
                    let _ = self.run("text.set", serde_json::json!({"id": 2, "text": bullets.join("  •  ")}));
                }
            }
            let bg = bg_color.unwrap_or("#0F172A");
            let _ = self.run("design.background", serde_json::json!({"color": bg}));
            let _ = self.run("shape.insert", serde_json::json!({"preset": "roundRect", "rect": [82.0, 142.0, 110.0, 6.0], "text": ""}));
            let _ = self.run("shape.fill", serde_json::json!({"color": accent_color}));
            let _ = self.run("shape.line", serde_json::json!({"color": accent_color, "width": 0.0}));
        } else if !cards_arr.is_empty() || layout_lower.contains("card") || layout_lower.contains("kpi") || layout_lower.contains("metric") || layout_lower.contains("two") || layout_lower.contains("column") {
            let body_empty = "";
            let _ = self.run("slide.new", serde_json::json!({"layout": "titleOnly", "title": title, "body": body_empty}));
            if reuse_first {
                if let Some(d) = self.session.active() {
                    if d.doc.slides.len() >= 2 {
                        let _ = self.run("slide.go", serde_json::json!({"index": 0}));
                        let _ = self.run("slide.delete", serde_json::json!({}));
                    }
                }
            }
            let bg = bg_color.unwrap_or("#0F172A");
            let _ = self.run("design.background", serde_json::json!({"color": bg}));
            if !subtitle.is_empty() {
                let _ = self.run("insert.textBox", serde_json::json!({"rect": [64.0, 96.0, 832.0, 32.0], "text": subtitle}));
                let _ = self.run("format.size", serde_json::json!({"size": 15.0}));
                let _ = self.run("format.color", serde_json::json!({"color": "#94A3B8"}));
            }

            // Build card items from either `cards_arr` or `bullets`
            let mut card_items: Vec<(String, String, String)> = Vec::new();
            let palette = ["#38BDF8", "#818CF8", "#34D399", "#F472B6"];
            if !cards_arr.is_empty() {
                for (idx, c) in cards_arr.iter().take(4).enumerate() {
                    let c_title = c.get("title").or_else(|| c.get("heading")).or_else(|| c.get("label")).and_then(serde_json::Value::as_str)
                        .unwrap_or_else(|| c.as_str().unwrap_or("Kernpunkt"))
                        .to_string();
                    let c_body = if let Some(b) = c.get("body").or_else(|| c.get("text")).or_else(|| c.get("description")).or_else(|| c.get("value")).and_then(serde_json::Value::as_str) {
                        b.to_string()
                    } else if let Some(arr) = c.get("bullets").and_then(serde_json::Value::as_array) {
                        arr.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join("\n• ")
                    } else {
                        String::new()
                    };
                    let c_acc = c.get("accent").and_then(serde_json::Value::as_str).unwrap_or(palette[idx % palette.len()]).to_string();
                    card_items.push((c_title, c_body, c_acc));
                }
            } else {
                for (idx, b) in bullets.iter().take(3).enumerate() {
                    let (ct, cb) = if let Some((h, rest)) = b.split_once(':') {
                        (h.trim().trim_matches('*').to_string(), rest.trim().to_string())
                    } else {
                        (format!("Punkt {}", idx + 1), b.clone())
                    };
                    card_items.push((ct, cb, palette[idx % palette.len()].to_string()));
                }
            }

            let n = card_items.len().clamp(1, 4);
            let total_w = 832.0_f32;
            let gap = 22.0_f32;
            let card_w = (total_w - gap * (n as f32 - 1.0)) / (n as f32);
            let top_y = if subtitle.is_empty() { 142.0 } else { 154.0 };
            let card_h = 316.0;

            for (i, (ct, cb, c_acc)) in card_items.into_iter().enumerate() {
                let x = 64.0 + i as f32 * (card_w + gap);
                let _ = self.run("shape.insert", serde_json::json!({"preset": "roundRect", "rect": [x, top_y, card_w, card_h], "text": ""}));
                let _ = self.run("shape.fill", serde_json::json!({"color": "#1E293B"}));
                let _ = self.run("shape.line", serde_json::json!({"color": c_acc, "width": 1.5}));
                let _ = self.run("shape.effects", serde_json::json!({"shadow": true, "reflection": false, "glow": false, "softEdges": 0.0}));

                let _ = self.run("insert.textBox", serde_json::json!({"rect": [x + 18.0, top_y + 20.0, card_w - 36.0, 44.0], "text": ct}));
                let _ = self.run("format.size", serde_json::json!({"size": 19.0}));
                let _ = self.run("format.bold", serde_json::json!({}));
                let _ = self.run("format.color", serde_json::json!({"color": c_acc}));

                if !cb.is_empty() {
                    let _ = self.run("insert.textBox", serde_json::json!({"rect": [x + 18.0, top_y + 72.0, card_w - 36.0, card_h - 90.0], "text": cb}));
                    let _ = self.run("format.size", serde_json::json!({"size": 14.5}));
                    let _ = self.run("format.color", serde_json::json!({"color": "#E2E8F0"}));
                }
            }
        } else {
            let body_text = if !bullets.is_empty() {
                bullets.join("\n")
            } else {
                subtitle.to_string()
            };
            let engine_layout = match layout_lower.as_str() {
                "section" | "sectionheader" => "sectionHeader",
                "titleonly" => "titleOnly",
                "blank" => "blank",
                _ => "titleAndContent",
            };
            let _ = self.run("slide.new", serde_json::json!({
                "layout": engine_layout,
                "title": title,
                "body": body_text
            }));
            if reuse_first {
                if let Some(d) = self.session.active() {
                    if d.doc.slides.len() >= 2 {
                        let _ = self.run("slide.go", serde_json::json!({"index": 0}));
                        let _ = self.run("slide.delete", serde_json::json!({}));
                    }
                }
            }
            if let Some(bg) = bg_color {
                let _ = self.run("design.background", serde_json::json!({"color": bg}));
            }
        }

        if let Some(notes) = spec.get("notes").and_then(serde_json::Value::as_str) {
            if !notes.trim().is_empty() {
                let _ = self.run("notes.set", serde_json::json!({"text": notes}));
            }
        }

        let slide_count = self.session.active().map(|d| d.doc.slides.len()).unwrap_or(1);
        Ok(serde_json::json!({
            "status": "ok",
            "title": title,
            "layout": raw_layout,
            "slide_count": slide_count
        }))
    }

    /// Polls and executes AI Agent tool calls on the main thread for DeckCraft.
    pub fn poll_artcraft_agent(&mut self) {
        let reqs = self.artcraft_agent.take_tool_requests();
        for req in reqs {
            let res = match req.tool_name.as_str() {
                "deck.generatePresentation" | "generate_presentation" => {
                    let theme_name = req.params.get("theme").and_then(serde_json::Value::as_str).unwrap_or("Nocturne");
                    let replace = req.params.get("replace").and_then(serde_json::Value::as_bool).unwrap_or(true);
                    if self.session.active().is_none() || replace {
                        let _ = self.run("file.new", serde_json::json!({"theme": theme_name}));
                    }
                    self.backstage = None;
                    self.sync_backstage();

                    let pres_title = req.params.get("title").and_then(serde_json::Value::as_str).unwrap_or("Präsentation");
                    let pres_sub = req.params.get("subtitle").and_then(serde_json::Value::as_str).unwrap_or("Erstellt mit DeckCraft AI Co-Creator");
                    let slides = req.params.get("slides").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();

                    if slides.is_empty() {
                        let _ = self.add_rich_slide_from_spec(&serde_json::json!({
                            "layout": "title",
                            "title": pres_title,
                            "subtitle": pres_sub,
                            "background": "#0F172A"
                        }));
                    } else {
                        for (idx, s) in slides.iter().enumerate() {
                            if idx == 0 && s.get("layout").and_then(serde_json::Value::as_str).unwrap_or("title").eq_ignore_ascii_case("title") {
                                let _ = self.add_rich_slide_from_spec(s);
                            } else {
                                if idx == 0 && self.is_only_slide_blank() {
                                    let _ = self.add_rich_slide_from_spec(&serde_json::json!({
                                        "layout": "title",
                                        "title": pres_title,
                                        "subtitle": pres_sub,
                                        "background": "#0F172A"
                                    }));
                                }
                                let _ = self.add_rich_slide_from_spec(s);
                            }
                        }
                    }
                    let _ = self.run("slide.go", serde_json::json!({"index": 0}));
                    let total = self.session.active().map(|d| d.doc.slides.len()).unwrap_or(1);
                    Ok(serde_json::json!({"status": "ok", "slides_created": total, "title": pres_title}))
                }
                "deck.addRichSlide" | "add_slide" | "slide.insert" | "slide.new" => {
                    self.add_rich_slide_from_spec(&req.params)
                }
                "deck.updateSlide" => {
                    self.ensure_active_presentation_for_agent(None);
                    if let Some(idx) = req.params.get("index").or_else(|| req.params.get("slide")).and_then(serde_json::Value::as_u64) {
                        let zero_idx = if idx > 0 { (idx as usize).saturating_sub(1) } else { 0 };
                        let _ = self.run("slide.go", serde_json::json!({"index": zero_idx}));
                    }
                    if let Some(bg) = req.params.get("background").or_else(|| req.params.get("bgColor")).and_then(serde_json::Value::as_str) {
                        let _ = self.run("design.background", serde_json::json!({"color": bg}));
                    }
                    if let Some(title) = req.params.get("title").and_then(serde_json::Value::as_str) {
                        let _ = self.run("text.set", serde_json::json!({"id": 1, "text": title}));
                    }
                    let bullets = Self::extract_bullets_from_value(&req.params);
                    if !bullets.is_empty() {
                        let _ = self.run("text.set", serde_json::json!({"id": 2, "text": bullets.join("\n")}));
                    }
                    Ok(serde_json::json!({"status": "ok"}))
                }
                "deck.deleteSlide" | "slide.delete" => {
                    self.ensure_active_presentation_for_agent(None);
                    if let Some(idx) = req.params.get("index").or_else(|| req.params.get("slide")).and_then(serde_json::Value::as_u64) {
                        let zero_idx = if idx > 0 { (idx as usize).saturating_sub(1) } else { 0 };
                        let _ = self.run("slide.go", serde_json::json!({"index": zero_idx}));
                    }
                    self.run("slide.delete", serde_json::json!({}))
                }
                "deck.applyTheme" | "design.theme" => {
                    let theme_name = req.params.get("theme").or_else(|| req.params.get("name")).and_then(serde_json::Value::as_str).unwrap_or("Nocturne");
                    self.ensure_active_presentation_for_agent(Some(theme_name));
                    let r = self.run("design.theme", serde_json::json!({"name": theme_name}));
                    if let Some(bg) = req.params.get("background").and_then(serde_json::Value::as_str) {
                        let _ = self.run("design.background", serde_json::json!({"color": bg}));
                    }
                    r
                }
                "deck.addTextElement" | "insert.textBox" => {
                    self.ensure_active_presentation_for_agent(None);
                    let text = req.params.get("text").and_then(serde_json::Value::as_str).unwrap_or("");
                    let x = req.params.get("x").and_then(serde_json::Value::as_f64).unwrap_or(80.0);
                    let y = req.params.get("y").and_then(serde_json::Value::as_f64).unwrap_or(140.0);
                    let w = req.params.get("w").or_else(|| req.params.get("width")).and_then(serde_json::Value::as_f64).unwrap_or(450.0);
                    let h = req.params.get("h").or_else(|| req.params.get("height")).and_then(serde_json::Value::as_f64).unwrap_or(80.0);
                    let rect = req.params.get("rect").cloned().unwrap_or_else(|| serde_json::json!([x, y, w, h]));
                    let r = self.run("insert.textBox", serde_json::json!({"rect": rect, "text": text}));
                    if let Some(sz) = req.params.get("fontSize").or_else(|| req.params.get("size")).and_then(serde_json::Value::as_f64) {
                        let _ = self.run("format.size", serde_json::json!({"size": sz}));
                    }
                    if req.params.get("bold").and_then(serde_json::Value::as_bool) == Some(true) {
                        let _ = self.run("format.bold", serde_json::json!({}));
                    }
                    if let Some(col) = req.params.get("color").and_then(serde_json::Value::as_str) {
                        let _ = self.run("format.color", serde_json::json!({"color": col}));
                    }
                    r
                }
                "deck.addShape" | "shape.insert" => {
                    self.ensure_active_presentation_for_agent(None);
                    let preset = req.params.get("preset").or_else(|| req.params.get("kind")).and_then(serde_json::Value::as_str).unwrap_or("roundRect");
                    let text = req.params.get("text").and_then(serde_json::Value::as_str).unwrap_or("");
                    let x = req.params.get("x").and_then(serde_json::Value::as_f64).unwrap_or(100.0);
                    let y = req.params.get("y").and_then(serde_json::Value::as_f64).unwrap_or(160.0);
                    let w = req.params.get("w").or_else(|| req.params.get("width")).and_then(serde_json::Value::as_f64).unwrap_or(280.0);
                    let h = req.params.get("h").or_else(|| req.params.get("height")).and_then(serde_json::Value::as_f64).unwrap_or(180.0);
                    let rect = req.params.get("rect").cloned().unwrap_or_else(|| serde_json::json!([x, y, w, h]));
                    let r = self.run("shape.insert", serde_json::json!({"preset": preset, "rect": rect, "text": text}));
                    if let Some(fill) = req.params.get("fill").or_else(|| req.params.get("color")).and_then(serde_json::Value::as_str) {
                        let _ = self.run("shape.fill", serde_json::json!({"color": fill}));
                    }
                    if let Some(stroke) = req.params.get("stroke").or_else(|| req.params.get("borderColor")).and_then(serde_json::Value::as_str) {
                        let _ = self.run("shape.line", serde_json::json!({"color": stroke, "width": 1.5}));
                    }
                    r
                }
                "insert_from_chat" => {
                    let md = req.params.get("markdown").or_else(|| req.params.get("text")).and_then(serde_json::Value::as_str).unwrap_or("");
                    let mut title = "KI-Folie".to_string();
                    let mut bullets = Vec::new();
                    for line in md.lines() {
                        let t = line.trim();
                        if t.is_empty() || t.starts_with("```") {
                            continue;
                        }
                        if let Some(h) = t.strip_prefix("# ").or_else(|| t.strip_prefix("## ")).or_else(|| t.strip_prefix("### ")) {
                            if title == "KI-Folie" {
                                title = h.trim().trim_matches('*').to_string();
                            } else {
                                bullets.push(h.trim().trim_matches('*').to_string());
                            }
                        } else if let Some(b) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")).or_else(|| t.strip_prefix("• ")) {
                            bullets.push(b.trim().replace("**", ""));
                        } else if !t.starts_with('|') {
                            bullets.push(t.replace("**", ""));
                        }
                    }
                    self.add_rich_slide_from_spec(&serde_json::json!({
                        "layout": if bullets.len() >= 2 && bullets.len() <= 4 { "cards" } else { "titleAndContent" },
                        "title": title,
                        "bullets": bullets,
                        "background": "#0F172A"
                    }))
                }
                "execute" => {
                    self.ensure_active_presentation_for_agent(None);
                    let cmd = req.params.get("command").and_then(serde_json::Value::as_str).unwrap_or("");
                    let params = req.params.get("params").cloned().unwrap_or(serde_json::json!({}));
                    match cmd {
                        "deck.generatePresentation" | "generate_presentation" => {
                            let theme_name = params.get("theme").and_then(serde_json::Value::as_str).unwrap_or("Nocturne");
                            let _ = self.run("file.new", serde_json::json!({"theme": theme_name}));
                            self.backstage = None;
                            self.sync_backstage();
                            if let Some(slides) = params.get("slides").and_then(serde_json::Value::as_array) {
                                for s in slides {
                                    let _ = self.add_rich_slide_from_spec(s);
                                }
                            }
                            let _ = self.run("slide.go", serde_json::json!({"index": 0}));
                            Ok(serde_json::json!({"status": "ok"}))
                        }
                        "deck.addRichSlide" | "add_slide" | "slide.insert" | "slide.new" => {
                            self.add_rich_slide_from_spec(&params)
                        }
                        "deck.applyTheme" => {
                            let th = params.get("theme").or_else(|| params.get("name")).and_then(serde_json::Value::as_str).unwrap_or("Nocturne");
                            self.run("design.theme", serde_json::json!({"name": th}))
                        }
                        "text.insert" | "type_text" => {
                            let txt = params.get("text").and_then(serde_json::Value::as_str).unwrap_or("");
                            self.run("insert.textBox", serde_json::json!({"rect": [80.0, 140.0, 600.0, 120.0], "text": txt}))
                        }
                        other_cmd => self.run(other_cmd, params),
                    }
                }
                "inspect_presentation" | "inspect_document" | "get_text" => {
                    let title = self.session.active().map(|d| d.title()).unwrap_or_else(|| "Unbenannt".into());
                    let dirty = self.session.active().map(|d| d.is_dirty()).unwrap_or(false);
                    let slide_count = self.session.active().map(|d| d.doc.slides.len()).unwrap_or(0);
                    let path = self.session.active().and_then(|d| d.path.clone());
                    let slides_info: Vec<serde_json::Value> = self.session.active().map(|d| {
                        d.doc.slides.iter().enumerate().map(|(i, s)| {
                            let texts: Vec<String> = s.shapes.iter().filter_map(|sh| sh.text.as_ref()).map(|tb| tb.paragraphs.iter().map(|p| p.text()).collect::<Vec<_>>().join("\n").trim().to_string()).filter(|t| !t.is_empty()).collect();
                            serde_json::json!({"slide": i + 1, "texts": texts})
                        }).collect()
                    }).unwrap_or_default();
                    Ok(serde_json::json!({
                        "title": title,
                        "dirty": dirty,
                        "slide_count": slide_count,
                        "slides": slides_info,
                        "path": path,
                    }))
                }
                "save_document" => {
                    let target = if let Some(p) = req.params.get("path").and_then(serde_json::Value::as_str) {
                        std::path::PathBuf::from(p)
                    } else if let Some(ref path) = self.session.active().and_then(|d| d.path.clone()) {
                        std::path::PathBuf::from(path)
                    } else {
                        self.artcraft_drive.set_or_init_project(None)
                    };
                    let r = self.run("file.saveAs", serde_json::json!({"path": target.to_string_lossy().to_string()}));
                    if r.is_ok() {
                        self.artcraft_drive.mark_saved();
                    }
                    r
                }
                other => self.run(other, req.params.clone()),
            };
            let _ = req.reply_sender.send(res);
        }

        self.artcraft_agent.process_events();
    }

    /// Automatically saves changes into the active Google Drive project folder if enabled.
    pub fn artcraft_autosave_tick(&mut self) {
        let is_dirty = self.session.active().map(|d| d.is_dirty()).unwrap_or(false);
        if self.artcraft_drive.auto_save_enabled && is_dirty && self.artcraft_drive.should_autosave() {
            let target = if let Some(ref path) = self.session.active().and_then(|d| d.path.clone()) {
                std::path::PathBuf::from(path)
            } else {
                self.artcraft_drive.set_or_init_project(None)
            };
            let target_str = target.to_string_lossy().to_string();
            if self.run("file.saveAs", serde_json::json!({"path": target_str})).is_ok() {
                self.artcraft_drive.mark_saved();
                self.set_status(format!("☁️ Gespeichert in {}", self.artcraft_drive.active_project_folder_name()));
            }
        }
    }
}
