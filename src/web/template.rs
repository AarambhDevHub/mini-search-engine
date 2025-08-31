pub const SEARCH_PAGE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Mini Search Engine</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }

        .header {
            background: rgba(255, 255, 255, 0.1);
            padding: 20px 0;
            text-align: center;
            backdrop-filter: blur(10px);
            border-bottom: 1px solid rgba(255, 255, 255, 0.2);
        }

        .header h1 {
            color: white;
            font-size: 2.5rem;
            margin-bottom: 10px;
            font-weight: 300;
        }

        .header p {
            color: rgba(255, 255, 255, 0.8);
            font-size: 1.1rem;
        }

        .search-container {
            max-width: 800px;
            margin: 50px auto;
            padding: 0 20px;
        }

        .search-box {
            position: relative;
            margin-bottom: 30px;
        }

        .search-input {
            width: 100%;
            padding: 15px 50px 15px 20px;
            font-size: 18px;
            border: none;
            border-radius: 50px;
            background: rgba(255, 255, 255, 0.9);
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
            transition: all 0.3s ease;
        }

        .search-input:focus {
            outline: none;
            background: rgba(255, 255, 255, 0.95);
            box-shadow: 0 8px 35px rgba(0, 0, 0, 0.15);
            transform: translateY(-2px);
        }

        .search-button {
            position: absolute;
            right: 5px;
            top: 50%;
            transform: translateY(-50%);
            background: #667eea;
            border: none;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.3s ease;
        }

        .search-button:hover {
            background: #5a6fd8;
            transform: translateY(-50%) scale(1.05);
        }

        .search-button svg {
            width: 20px;
            height: 20px;
            fill: white;
        }

        .stats {
            text-align: center;
            color: rgba(255, 255, 255, 0.8);
            margin-bottom: 20px;
            font-size: 0.9rem;
        }

        .results {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 15px;
            padding: 20px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
        }

        .result-item {
            padding: 20px 0;
            border-bottom: 1px solid #eee;
            transition: all 0.2s ease;
        }

        .result-item:hover {
            background: rgba(103, 126, 234, 0.05);
            padding: 20px 10px;
            border-radius: 10px;
        }

        .result-item:last-child {
            border-bottom: none;
        }

        .result-title {
            font-size: 1.4rem;
            font-weight: 600;
            color: #667eea;
            margin-bottom: 5px;
            cursor: pointer;
            text-decoration: none;
            display: block;
        }

        .result-title:hover {
            text-decoration: underline;
        }

        .result-url {
            color: #28a745;
            font-size: 0.9rem;
            margin-bottom: 8px;
            word-break: break-all;
        }

        .result-snippet {
            color: #555;
            line-height: 1.6;
            font-size: 1rem;
        }

        .result-score {
            float: right;
            background: rgba(103, 126, 234, 0.1);
            color: #667eea;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.8rem;
            font-weight: 600;
        }

        .loading {
            text-align: center;
            padding: 40px;
            color: rgba(255, 255, 255, 0.8);
        }

        .no-results {
            text-align: center;
            padding: 40px;
            color: #666;
        }

        .no-results h3 {
            margin-bottom: 10px;
            color: #667eea;
        }

        .example-queries {
            text-align: center;
            margin-top: 30px;
        }

        .example-queries h4 {
            color: rgba(255, 255, 255, 0.9);
            margin-bottom: 15px;
        }

        .example-query {
            display: inline-block;
            margin: 5px 10px;
            padding: 8px 16px;
            background: rgba(255, 255, 255, 0.2);
            color: white;
            border-radius: 20px;
            cursor: pointer;
            transition: all 0.2s ease;
            font-size: 0.9rem;
        }

        .example-query:hover {
            background: rgba(255, 255, 255, 0.3);
            transform: scale(1.05);
        }

        @media (max-width: 768px) {
            .search-container {
                margin: 30px auto;
                padding: 0 15px;
            }

            .header h1 {
                font-size: 2rem;
            }

            .search-input {
                font-size: 16px;
                padding: 12px 45px 12px 15px;
            }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 Mini Search Engine</h1>
        <p>Powered by Rust • TF-IDF Ranking • Web & Local Files</p>
    </div>

    <div class="search-container">
        <div class="search-box">
            <input
                type="text"
                class="search-input"
                id="searchInput"
                placeholder="Search for anything..."
                autocomplete="off"
            >
            <button class="search-button" id="searchButton" onclick="performSearch()">
                <svg viewBox="0 0 24 24">
                    <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                </svg>
            </button>
        </div>

        <div id="stats" class="stats" style="display: none;"></div>
        <div id="results" class="results" style="display: none;"></div>
        <div id="loading" class="loading" style="display: none;">
            <p>🔄 Searching...</p>
        </div>

        <div class="example-queries">
            <h4>Try these example searches:</h4>
            <div class="example-query" onclick="searchExample('rust programming')">rust programming</div>
            <div class="example-query" onclick="searchExample('performance')">performance</div>
            <div class="example-query" onclick="searchExample('memory safety')">memory safety</div>
            <div class="example-query" onclick="searchExample('web development')">web development</div>
        </div>
    </div>

    <script>
        const searchInput = document.getElementById('searchInput');
        const searchButton = document.getElementById('searchButton');
        const resultsDiv = document.getElementById('results');
        const statsDiv = document.getElementById('stats');
        const loadingDiv = document.getElementById('loading');

        // Search on Enter key
        searchInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                performSearch();
            }
        });

        async function performSearch() {
            const query = searchInput.value.trim();
            if (!query) return;

            // Show loading state
            showLoading();

            try {
                const response = await fetch(`/search?q=${encodeURIComponent(query)}&limit=10`);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }

                const data = await response.json();
                displayResults(data);
            } catch (error) {
                console.error('Search error:', error);
                showError('Failed to perform search. Please try again.');
            }
        }

        function showLoading() {
            resultsDiv.style.display = 'none';
            statsDiv.style.display = 'none';
            loadingDiv.style.display = 'block';
        }

        function displayResults(data) {
            loadingDiv.style.display = 'none';

            if (data.results && data.results.length > 0) {
                // Show stats
                statsDiv.innerHTML = `Found ${data.total} result${data.total !== 1 ? 's' : ''} for "${data.query}"`;
                statsDiv.style.display = 'block';

                // Show results
                let resultsHTML = '';
                data.results.forEach((result, index) => {
                    const isUrl = result.path.startsWith('http://') || result.path.startsWith('https://');
                    const linkTarget = isUrl ? '_blank' : '_self';
                    const linkHref = isUrl ? result.path : '#';

                    resultsHTML += `
                        <div class="result-item">
                            <div class="result-score">${result.score.toFixed(4)}</div>
                            <a href="${linkHref}" target="${linkTarget}" class="result-title">
                                ${escapeHtml(result.title)}
                            </a>
                            <div class="result-url">${escapeHtml(result.path)}</div>
                            <div class="result-snippet">${escapeHtml(result.snippet)}</div>
                        </div>
                    `;
                });

                resultsDiv.innerHTML = resultsHTML;
                resultsDiv.style.display = 'block';
            } else {
                // No results
                statsDiv.style.display = 'none';
                resultsDiv.innerHTML = `
                    <div class="no-results">
                        <h3>No results found</h3>
                        <p>Try different keywords or check your spelling.</p>
                    </div>
                `;
                resultsDiv.style.display = 'block';
            }
        }

        function showError(message) {
            loadingDiv.style.display = 'none';
            statsDiv.style.display = 'none';
            resultsDiv.innerHTML = `
                <div class="no-results">
                    <h3>Error</h3>
                    <p>${escapeHtml(message)}</p>
                </div>
            `;
            resultsDiv.style.display = 'block';
        }

        function searchExample(query) {
            searchInput.value = query;
            performSearch();
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Focus search input on page load
        window.addEventListener('load', function() {
            searchInput.focus();
        });
    </script>
</body>
</html>"#;
