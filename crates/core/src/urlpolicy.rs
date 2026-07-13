//! URL 白名单策略(Plan 34 S3,shadcn/Streamdown R12;0006/0007 策略节)。
//!
//! 模型吐出的链接/图片 URL 在**派发/加载前**过 prefix 白名单:默认仅 `https:`/`http:`/
//! `data:image/`,拒 `javascript:`/`file:`/一切未知协议。纯函数(CR1),恶意样本不 panic
//! (AR12);拒绝路径的视觉/行为降级由调用方(app/embed)执行。

/// URL 白名单(链接与图片各一份 prefix 表;比较前做规范化)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPolicy {
    link_prefixes: Vec<String>,
    image_prefixes: Vec<String>,
    /// 相对 URL(无 scheme,`/` 或 `./` 开头)解析基(Streamdown `defaultOrigin`);
    /// None = 相对 URL 一律拒(默认,最保守)。
    default_origin: Option<String>,
}

impl Default for UrlPolicy {
    fn default() -> Self {
        Self {
            link_prefixes: vec!["https:".into(), "http:".into(), "data:image/".into()],
            image_prefixes: vec!["https:".into(), "http:".into(), "data:image/".into()],
            default_origin: None,
        }
    }
}

impl UrlPolicy {
    /// 自定义表(宿主 setter;空表 = 全拒)。
    #[must_use]
    pub fn new(
        link_prefixes: Vec<String>,
        image_prefixes: Vec<String>,
        default_origin: Option<String>,
    ) -> Self {
        Self {
            link_prefixes,
            image_prefixes,
            default_origin,
        }
    }

    /// 链接可打开?(tap 派发前的守门员)
    #[must_use]
    pub fn link_allowed(&self, url: &str) -> bool {
        allowed(url, &self.link_prefixes, self.default_origin.as_deref())
    }

    /// 图片可加载?(embed loader 前的守门员)
    #[must_use]
    pub fn image_allowed(&self, url: &str) -> bool {
        allowed(url, &self.image_prefixes, self.default_origin.as_deref())
    }

    /// 链接前缀表副本(宿主 setter 与默认表合并用)。
    #[must_use]
    pub fn link_prefixes_vec(&self) -> Vec<String> {
        self.link_prefixes.clone()
    }

    /// 图片前缀表副本(同上)。
    #[must_use]
    pub fn image_prefixes_vec(&self) -> Vec<String> {
        self.image_prefixes.clone()
    }
}

/// 规范化 + prefix 匹配。规范化(对齐浏览器解析的宽容面,堵绕过):
/// - 去首尾 ASCII 空白;剥 URL 内的 `\t`/`\n`/`\r`(浏览器解析时忽略 → `java\tscript:` 绕过);
/// - scheme 比较大小写不敏感(`JAVASCRIPT:`);
/// - 相对 URL(无 `:` 或 `:` 前含 `/`)→ 有 default_origin 才放行(交宿主解析),否则拒。
fn allowed(url: &str, prefixes: &[String], default_origin: Option<&str>) -> bool {
    let cleaned: String = url
        .trim()
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    if cleaned.is_empty() {
        return false;
    }
    // 相对 URL:`:` 之前出现 `/`(路径)或全串无 `:` → 无协议。
    let has_scheme = match cleaned.find(':') {
        Some(i) => !cleaned[..i].contains('/') && i > 0,
        None => false,
    };
    if !has_scheme {
        return default_origin.is_some();
    }
    let lower = cleaned.to_ascii_lowercase();
    prefixes
        .iter()
        .any(|p| lower.starts_with(&p.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 34 S3:协议矩阵 —— 恶意样本全拒、白名单内放行,全程不 panic(AR12)。
    #[test]
    fn protocol_matrix_rejects_hostile_allows_whitelisted() {
        let p = UrlPolicy::default();
        // 放行。
        for ok in [
            "https://example.com/a",
            "http://localhost:4096/x",
            "HTTPS://UPPER.CASE/PATH",
            "data:image/png;base64,iVBORw0KGgo=",
            "  https://trimmed.example  ",
        ] {
            assert!(p.link_allowed(ok), "应放行: {ok}");
            assert!(p.image_allowed(ok), "应放行: {ok}");
        }
        // 拒绝(协议注入/本地/未知/绕过尝试)。
        for bad in [
            "javascript:alert(1)",
            "JAVASCRIPT:alert(1)",
            "java\tscript:alert(1)",
            " \n javascript:alert(1)",
            "vbscript:msgbox(1)",
            "file:///etc/passwd",
            "data:text/html,<script>1</script>",
            "chrome://settings",
            "about:blank",
            "blob:https://x",
            "//protocol-relative.example", // 无 default_origin → 拒
            "/relative/path",
            "ftp://old.example",
            "",
            "   ",
            ":no-scheme",
        ] {
            assert!(!p.link_allowed(bad), "应拒绝: {bad}");
            assert!(!p.image_allowed(bad), "应拒绝: {bad}");
        }
    }

    /// default_origin 放行相对 URL(解析交宿主);自定义表生效;空表全拒。
    #[test]
    fn default_origin_and_custom_tables() {
        let p = UrlPolicy::new(
            vec!["https://trusted.example/".into()],
            vec![],
            Some("https://site.example".into()),
        );
        assert!(p.link_allowed("https://trusted.example/doc"));
        assert!(!p.link_allowed("https://other.example/doc"), "prefix 外拒");
        assert!(p.link_allowed("/relative/ok"), "有 origin → 相对放行");
        assert!(!p.image_allowed("https://trusted.example/img"), "空表全拒");
        // 畸形长输入不 panic。
        let long = "a".repeat(100_000) + ":x";
        let _ = p.link_allowed(&long);
    }
}
