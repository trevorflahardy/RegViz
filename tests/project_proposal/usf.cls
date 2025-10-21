\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{usf}
\LoadClass[11pt, letterpaper, onecolumn]{article}

%%
%% Set PDF font encodings. Must install `cm-super` for MikTex to allow for
%%      anti-alias fonts.
%%
\RequirePackage[T1]{fontenc} % must manually install `cm-super` if in MikTex
%\RequirePackage{cmap}

%%
%% Set page margins and cross-references (Most are hard required and should
%%      not be changed.)
%%
\RequirePackage[margin=1in]{geometry}
\RequirePackage[unicode,pdftex,colorlinks,linkcolor=black,citecolor=black,urlcolor=black,filecolor=black,pdfauthor=author]{hyperref}

%%
%% Set allowable figure formats. I prefer PNG to JPG, and PDF for the best quality.
%%
\RequirePackage{graphicx}
% \DeclareGraphicsExtensions{.eps,.pdf,.jpeg,.png}
\DeclareGraphicsExtensions{.pdf,.png}
\graphicspath{{figs/}}

%%
%% Set page number styles and locations. Change to whatever you prefer.
%%
\RequirePackage{fancyhdr}
\pagestyle{fancyplain}
\renewcommand{\headrulewidth}{0pt}
\fancyhf{}
%\fancyfoot[C]{\thepage}
\fancyhead[C]{\itshape\footnotesize\proposalTitle}
% Reset page numbering to 1.  This is helpful, since the text can only be 15 pages (unless otherwise specified, see individual solicitations), and reviewers will want to believe we've kept it within those limits
\newcommand{\resetSectionPageNumber}[2][]{
	\pagenumbering{arabic}
	\setcounter{page}{1}
	\fancyfoot[R]{\textcolor{gray}{\footnotesize #2 | Page \thepage{} #1}}
}

%%
%% Set heading styles. Change to whatever you prefer.
%%
\RequirePackage{fancybox} % provide `\ovalbox` etc.
% \RequirePackage{titlecaps} % Capitalize all chars
\RequirePackage[explicit]{titlesec}
% `titlesec` syntax:
%   - \titleformat{⟨command ⟩}[⟨shape⟩]{⟨format⟩}{⟨label ⟩}{⟨sep⟩}{⟨before-code⟩}[⟨after-code⟩]
%   - \titlespacing{command}{left spacing}{before spacing}{after spacing}[right]
\titleformat{\section}{\Large\bfseries\filcenter}{}{0pt}{{#1}\\\vspace{-1ex}\rule{0.82\textwidth}{0.4pt}}
\titlespacing*{\section}{0pt}{1ex}{0.5ex}
%
\titleformat{\subsection}{\Large\bfseries}{}{0pt}{{#1}}
\titlespacing*{\subsection}{0pt}{1ex}{0.5ex}
%
\titleformat{\subsubsection}{\bfseries}{}{0pt}{#1}
\titlespacing*{\subsubsection}{0pt}{0.75ex}{0.5ex}
%
\titleformat{\paragraph}[runin]{\bfseries}{}{0pt}{{#1}}
\titlespacing*{\paragraph}{0pt}{0.75ex}{2\wordsep}

%%
%% Set bibliography. `Bibtex` is used. `Biblatex` is NOT supported.
%%      Because `bibentry` is used to separate bibliography in a different file.
%%
\RequirePackage[square,sort,comma,numbers]{natbib} % support `bibentry`
\bibliographystyle{unsrtnat} % let bibliography show in the appearance order in context
\RequirePackage{bibentry} % use `\nobibliography` to suppress bibliography at the end.


%---------------------------------------------------------------
% Some other commonly used packages.
%   These used to be in the main preamble, but Peng moved them here to keep the
%   source files as clean and neat as possible.
%   Feel free to add or change any packages below.
%---------------------------------------------------------------

\RequirePackage{float}
\RequirePackage{dcolumn}% Align table columns on decimal point
\RequirePackage{amsmath}% bold math
\RequirePackage{bm}% bold math
\RequirePackage{amssymb}
% \RequirePackage{booktabs}
% \RequirePackage{boldline}
%\RequirePackage{algorithmicx}
%\RequirePackage{algorithm}% http://ctan.org/pkg/algorithms
%\RequirePackage{algpseudocode}% http://ctan.org/pkg/algorithmicx
%\def\NoNumber#1{{\def\alglinenumber##1{}\State #1}\addtocounter{ALG@line}{-1}}
\RequirePackage{caption}
\RequirePackage{blindtext}
\RequirePackage{tcolorbox}
% \RequirePackage[final]{pdfpages}
%\RequirePackage{lipsum,multicol}
%\RequirePackage{xcolor}
%\RequirePackage{tikz}
%\RequirePackage{listings}
\RequirePackage[normalem]{ulem}
\RequirePackage{enumitem} % to alter list formats
\RequirePackage{placeins} % to use `\FloatBarrier`
% \RequirePackage{wrapfig} % to wrap text around figures or tables
\RequirePackage{longtable} % allow tables spanning through pages
\usepackage{lipsum} % Generate sample text