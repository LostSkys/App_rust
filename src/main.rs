use axum::{
    routing::get,
    Router,
    http::{StatusCode, Request, header},
    response::{IntoResponse, Html},
    middleware::{self, Next},
    body::Body,
};
use tower_http::services::ServeDir;
use base64::{engine::general_purpose, Engine as _};
use std::{fs, env};

#[tokio::main]
async fn main() {
    // --- Lecture des variables d'environnement ---
    let login = env::var("RUST_LOGIN").unwrap_or_else(|_| "admin".to_string());
    let mdp   = env::var("RUST_PASSWORD").unwrap_or_else(|_| "changeme".to_string());

    let dossier_perso      = env::var("UPLOAD_PERSO")
        .unwrap_or_else(|_| "/uploads/perso".to_string());
    let dossier_commandes  = env::var("UPLOAD_COMMANDES")
        .unwrap_or_else(|_| "/uploads/commandes".to_string());

    let auth_valide = format!("{}:{}", login, mdp);
    let expected = format!("Basic {}", general_purpose::STANDARD.encode(auth_valide));

    // Clones pour les closures
    let expected_clone      = expected.clone();
    let dossier_perso_1     = dossier_perso.clone();
    let dossier_commandes_1 = dossier_commandes.clone();
    let dossier_perso_2     = dossier_perso.clone();
    let dossier_commandes_2 = dossier_commandes.clone();

    let app = Router::new()

        // --- Page d'accueil : choix du site ---
        .route("/", get(|| async {
            Html(String::from(r#"
                <html>
                <head>
                    <meta charset="UTF-8">
                    <style>
                        * { box-sizing: border-box; margin: 0; padding: 0; }
                        body { font-family: sans-serif; background: #f4f4f4; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; }
                        h1 { color: #333; margin-bottom: 40px; font-size: 1.8rem; }
                        .cards { display: flex; gap: 30px; }
                        .card { background: white; border-radius: 12px; padding: 40px 50px; text-align: center; text-decoration: none; color: #333; box-shadow: 0 4px 15px rgba(0,0,0,0.1); transition: transform 0.2s, box-shadow 0.2s; }
                        .card:hover { transform: translateY(-4px); box-shadow: 0 8px 25px rgba(0,0,0,0.15); }
                        .card .icon { font-size: 3rem; margin-bottom: 15px; }
                        .card h2 { font-size: 1.2rem; }
                        .card p { font-size: 0.85rem; color: #888; margin-top: 8px; }
                    </style>
                </head>
                <body>
                    <h1>📁 Espace Fichiers CDHV</h1>
                    <div class="cards">
                        <a class="card" href="/perso/">
                            <div class="icon">🎨</div>
                            <h2>Personnalisation</h2>
                            <p>Visuels clients (recto/verso)</p>
                        </a>
                        <a class="card" href="/commandes/">
                            <div class="icon">📦</div>
                            <h2>Commandes Groupées</h2>
                            <p>PDFs des commandes associations</p>
                        </a>
                    </div>
                </body>
                </html>
            "#))
        }))

        // --- Galerie Personnalisation ---
        .route("/perso/", get(move || {
            let dossier = dossier_perso_1.clone();
            async move { generer_page_galerie(&dossier, "Personnalisation", "perso") }
        }))
        .nest_service("/perso/src", ServeDir::new(dossier_perso_2))

        // --- Galerie Commandes ---
        .route("/commandes/", get(move || {
            let dossier = dossier_commandes_1.clone();
            async move { generer_page_galerie(&dossier, "Commandes Groupées", "commandes") }
        }))
        .nest_service("/commandes/src", ServeDir::new(dossier_commandes_2))

        // --- Middleware auth basique sur toutes les routes ---
        .layer(middleware::from_fn(move |req: Request<Body>, next: Next| {
            let expected_clone = expected_clone.clone();
            let auth_header = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .map(String::from);

            async move {
                if let Some(header_val) = auth_header {
                    if header_val == expected_clone {
                        return next.run(req).await;
                    }
                }
                (
                    StatusCode::UNAUTHORIZED,
                    [(header::WWW_AUTHENTICATE, r#"Basic realm="Espace CDHV""#)],
                    "Accès refusé",
                ).into_response()
            }
        }));

    println!("Galerie lancée sur http://0.0.0.0:8090");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- Génération de la page galerie ---
fn generer_page_galerie(chemin: &str, titre: &str, section: &str) -> Html<String> {
    let mut fichiers = Vec::new();

    if let Ok(entries) = fs::read_dir(chemin) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().unwrap();
                let modif = metadata.modified().unwrap();
                let nom = entry.file_name().into_string().unwrap_or_default();

                // Images pour la perso, PDFs pour les commandes
                let est_valide = nom.ends_with(".jpg")
                    || nom.ends_with(".png")
                    || nom.ends_with(".jpeg")
                    || nom.ends_with(".pdf");

                if est_valide {
                    fichiers.push((nom, modif));
                }
            }
        }
    }

    fichiers.sort_by(|a, b| b.1.cmp(&a.1));

    let mut html = format!(r#"
        <html>
        <head>
            <meta charset="UTF-8">
            <style>
                * {{ box-sizing: border-box; }}
                body {{ font-family: sans-serif; background: #f4f4f4; padding: 20px; }}
                .header {{ display: flex; align-items: center; gap: 20px; margin-bottom: 30px; }}
                .header a {{ text-decoration: none; color: #666; font-size: 0.9rem; }}
                .header a:hover {{ color: #333; }}
                h1 {{ color: #333; }}
                .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px; }}
                .card {{ background: white; padding: 10px; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); text-align: center; }}
                img {{ max-width: 100%; border-radius: 4px; height: 150px; object-fit: cover; }}
                .pdf-icon {{ font-size: 3rem; height: 150px; display: flex; align-items: center; justify-content: center; }}
                .tag {{ font-size: 0.8em; padding: 3px 8px; border-radius: 10px; color: white; margin-bottom: 5px; display: inline-block; }}
                .recto {{ background: #e67e22; }} .verso {{ background: #3498db; }}
                .image {{ background: #2ecc71; }} .pdf {{ background: #9b59b6; }}
                .empty {{ color: #999; text-align: center; padding: 60px; font-size: 1.1rem; }}
            </style>
        </head>
        <body>
            <div class="header">
                <a href="/">← Retour</a>
                <h1>📁 {titre} ({} fichiers)</h1>
            </div>
            <div class="grid">
    "#, fichiers.len(), titre = titre);

    if fichiers.is_empty() {
        html.push_str(r#"<div class="empty">Aucun fichier disponible pour le moment.</div>"#);
    } else {
        for (nom, _) in fichiers {
            let est_pdf = nom.ends_with(".pdf");

            let categorie = if nom.contains("recto") { "recto" }
                            else if nom.contains("verso") { "verso" }
                            else if est_pdf { "pdf" }
                            else { "image" };

            let apercu = if est_pdf {
                format!(r#"<a href="/{}/src/{}" target="_blank"><div class="pdf-icon">📄</div></a>"#, section, nom)
            } else {
                format!(r#"<a href="/{}/src/{}" target="_blank"><img src="/{}/src/{}"></a>"#, section, nom, section, nom)
            };

            html.push_str(&format!(r#"
                <div class="card">
                    <span class="tag {}">{}</span><br>
                    {}
                    <p style="font-size: 10px; overflow-wrap: break-word; margin-top: 8px;">{}</p>
                </div>
            "#, categorie, categorie.to_uppercase(), apercu, nom));
        }
    }

    html.push_str("</div></body></html>");
    Html(html)
}