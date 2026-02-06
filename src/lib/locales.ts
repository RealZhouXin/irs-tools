import type { Language, Translation } from "../types";

export const textMap: Record<Language, Translation> = {
    zh: {
        title: "检测软件",
        subtitle: "点击开始检测后，将通过 DLL 调用测试指令并返回检测结果。",
        start: "开始检测",
        statusTitle: "检测状态",
        status: {
            idle: "等待开始",
            running: "检测中，请稍候...",
            done: "检测完成",
            failed: "检测失败",
        },
        summary: {
            pass: "全部通过",
            fail: "未通过",
            pending: "进行中",
            idle: "未开始",
        },
        table: {
            group: "检测项",
            command: "命令",
            range: "阈值范围",
            value: "检测值",
            result: "结果",
            empty: "尚未开始检测",
        },
        pass: "通过",
        fail: "未通过",
        retest: "重测",
        retesting: "重测中...",
        configTitle: "配置说明",
        configPrefix: "连接方式与超时在",
        configMiddle: "，测试项在",
        configSuffix: "中配置。",
        langLabel: "EN",
    },
    en: {
        title: "Test Console",
        subtitle: "Click Start to run DLL test commands and show the results.",
        start: "Start Test",
        statusTitle: "Status",
        status: {
            idle: "Ready",
            running: "Running...",
            done: "Completed",
            failed: "Failed",
        },
        summary: {
            pass: "All Pass",
            fail: "Failed",
            pending: "In Progress",
            idle: "Not Started",
        },
        table: {
            group: "Test Item",
            command: "Command",
            range: "Range",
            value: "Value",
            result: "Result",
            empty: "No tests started.",
        },
        pass: "Pass",
        fail: "Fail",
        retest: "Retest",
        retesting: "Retesting...",
        configTitle: "Configuration",
        configPrefix: "Connection settings and timeout are in",
        configMiddle: ", tests are in",
        configSuffix: ".",
        langLabel: "中文",
    },
};

export function getTranslation(lang: Language): Translation {
    return textMap[lang];
}
