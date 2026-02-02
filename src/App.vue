<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { 
  Home,
  LayoutDashboard, 
  History, 
  Search, 
  ExternalLink,
  ChevronRight,
  Loader2,
  RefreshCw,
  Plus,
  X,
  Send,
  Info,
  Copy,
  Check,
  ShieldCheck,
  ShieldAlert,
  KeyRound
} from "lucide-vue-next";

// State
const currentTab = ref('home'); // 'home', 'inquiry'
const token = ref('');
const isTokenExpired = ref(true); // 默认设置为 true，直到用户认证通过
const tokenExpiryMessage = ref("请先完成用户认证（更新 Token）以获取数据");

const loading = ref(false);
const query = ref("");
const historySearchQuery = ref(""); // New state for history filtering
const securityPool = ref<any[]>([]);
const showDropdown = ref(false);
const history = ref<any[]>([]);
const totalHistory = ref(0);
const pageSize = ref(100); // Set to a large number to effectively disable pagination from UI perspective
const currentPage = ref(1);
const createLoading = ref(false);
const inquiryProgress = ref(0);
const currentInquiryTarget = ref("");
const showInquirySuccess = ref(false);
const expandedRows = ref<string[]>([]);
const globalMarkup = ref<number | null>(1.0); // 全局加价，默认为 1.0

// Computed for filtered history
const filteredHistory = computed(() => {
  if (!historySearchQuery.value.trim()) return history.value;
  const q = historySearchQuery.value.toLowerCase().trim();
  return history.value.filter(item => 
    item.securityCode?.toLowerCase().includes(q) || 
    item.securityName?.toLowerCase().includes(q)
  );
});

const toggleRow = (id: string | number) => {
  const stringId = String(id);
  const index = expandedRows.value.indexOf(stringId);
  if (index > -1) {
    expandedRows.value.splice(index, 1);
  } else {
    expandedRows.value.push(stringId);
  }
};

const getRowId = (item: any, index: number) => {
  if (item.optInquiryBuyerId !== undefined && item.optInquiryBuyerId !== null && item.optInquiryBuyerId !== '') {
    return String(item.optInquiryBuyerId);
  }
  return `row-${index}`;
};

// Form State
const formData = reactive({
  securities: [] as any[], // Array of {code, name}
  notionalPrincipal: 500 as number | string, // In 万
  customNotional: null as number | null,
  dateLimits: [] as string[],
  structures: [] as string[],
  specialRequirements: [] as string[]
});

const scaleOptions = [100, 500, 1000, 'custom'];
const tenorRows = [
  [
    { label: '2周', value: 'W2' },
    { label: '1个月', value: 'M1' },
    { label: '2个月', value: 'M2' },
    { label: '3个月', value: 'M3' }
  ],
  [
    { label: '6个月', value: 'M6' },
    { label: '9个月', value: 'M9' },
    { label: '12个月', value: 'M12' }
  ]
];
const specialOptions = [
  { label: '协商敲出 (主3/创2)', value: 'tripleKnockout' },
  { label: 'T+1可行权', value: 'tOneExecute' },
  { label: '分红调整期初价', value: 'dividendAdjust' }
];
const structureRows = [
  ['S70', 'S80', 'S90', 'S95'],
  ['S100'],
  ['S103', 'S105', 'S110', 'S120']
];

const removeSecurity = (code: string) => {
  formData.securities = formData.securities.filter(s => s.securityCode !== code);
};

const toggleSelection = (list: string[], val: string) => {
  const idx = list.indexOf(val);
  if (idx > -1) {
    list.splice(idx, 1);
  } else {
    list.push(val);
  }
};

const selectSecurity = (item: any) => {
  if (!formData.securities.find(s => s.securityCode === item.securityCode)) {
    formData.securities.push({
      securityCode: item.securityCode,
      securityName: item.securityName
    });
  }
  query.value = '';
  showDropdown.value = false;
};

const handleInputSearch = async () => {
  if (!query.value.trim()) {
    showDropdown.value = false;
    securityPool.value = [];
    return;
  }
  
  if (isTokenExpired.value) {
    console.log("Token 已过期，跳过搜索");
    return;
  }
  
  try {
    const response: any = await invoke("fetch_price", { 
      query: query.value.trim(),
      token: token.value
    });
    
    // Tauri result success check
    if (response.success) {
      const pythonResult = response.data;
      if (pythonResult.status_code === 401 || (pythonResult.error && pythonResult.error.includes("401"))) {
        isTokenExpired.value = true;
        tokenExpiryMessage.value = "Token 已过期或无效，请在主页更新。";
      }
      // pythonResult is { code: 200, data: [...], success: true }
      if (pythonResult.success && Array.isArray(pythonResult.data)) {
        securityPool.value = pythonResult.data;
        showDropdown.value = true;
      } else {
        securityPool.value = [];
        showDropdown.value = true;
      }
    } else {
      console.error("API error:", response.error);
      securityPool.value = [];
      showDropdown.value = false;
    }
  } catch (error) {
    console.error("Security search failed:", error);
    securityPool.value = [];
    showDropdown.value = false;
  }
};



const fetchHistory = async (isAuto = false) => {
  // 如果 Token 为空或已过期，不执行请求
  if (!token.value || isTokenExpired.value) {
    if (!isAuto) console.log("Token 未设置或已过期，跳过获取历史记录");
    return;
  }

  if (!isAuto) loading.value = true;
  try {
    const response: any = await invoke("fetch_quote_list", { 
      page: currentPage.value, 
      pageSize: pageSize.value,
      securityCode: "",
      token: token.value
    });
    console.log("History response:", response);
    
    if (response.success) {
      const pythonResult = response.data;
      if (pythonResult.status_code === 401 || (pythonResult.error && pythonResult.error.includes("401"))) {
        isTokenExpired.value = true;
        tokenExpiryMessage.value = "Token 已过期或无效，请在主页更新。";
      }
      if (pythonResult && pythonResult.success) {
        history.value = pythonResult.data.list || [];
        totalHistory.value = pythonResult.data.total || 0;
      } else {
        const errorMsg = pythonResult?.msg || "API 返回失败";
        console.error("API Error:", errorMsg);
        if (!isAuto) alert("获取历史记录失败: " + errorMsg);
      }
    } else {
      console.error("Script Error:", response.error);
      if (!isAuto) alert("执行脚本失败: " + response.error);
    }
  } catch (error) {
    console.error("Failed to fetch history:", error);
    if (!isAuto) alert("网络或系统错误: " + error);
  } finally {
    if (!isAuto) loading.value = false;
  }
};

// Auto Refresh Logic
const autoRefreshTimer = ref<any>(null);
const isAutoRefreshing = ref(true);
const lastRefreshTime = ref(new Date());

const startAutoRefresh = () => {
  if (autoRefreshTimer.value) clearTimeout(autoRefreshTimer.value);
  
  // 随机刷新间隔：3秒到10秒之间
  const minInterval = 3000;
  const maxInterval = 10000;
  const randomInterval = Math.floor(Math.random() * (maxInterval - minInterval + 1)) + minInterval;
  
  autoRefreshTimer.value = setTimeout(async () => {
    if (isAutoRefreshing.value) {
      console.log(`Auto refreshing... next in ${randomInterval/1000}s`);
      await fetchHistory(true);
      lastRefreshTime.value = new Date();
    }
    startAutoRefresh(); // 递归调用以维持循环
  }, randomInterval);
};

onMounted(() => {
  fetchHistory();
  startAutoRefresh();
});

onUnmounted(() => {
  if (autoRefreshTimer.value) clearTimeout(autoRefreshTimer.value);
});

const chunkArray = <T>(arr: T[], size: number): T[][] => {
  const chunks: T[][] = [];
  for (let i = 0; i < arr.length; i += size) {
    chunks.push(arr.slice(i, i + size));
  }
  return chunks;
};

const handleCreate = async () => {
  if (formData.securities.length === 0 || formData.structures.length === 0 || formData.dateLimits.length === 0) {
    alert("请至少选择一个标的、一个结构和一种期限");
    return;
  }
  
  createLoading.value = true;
  inquiryProgress.value = 0;
  currentInquiryTarget.value = "准备提交询价请求...";
  showInquirySuccess.value = false;
  
  // 强制 UI 刷新
  await new Promise(resolve => setTimeout(resolve, 50));
  let successCount = 0;
  let failCount = 0;
  let lastError = "";

  const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  try {
    // 准备基础参数
    let notional = 500;
    if (formData.notionalPrincipal === 'custom') {
      notional = Number(formData.customNotional) || 500;
    } else {
      notional = Number(formData.notionalPrincipal);
    }

    // 分拆逻辑：标的每 5 个一组，结构每 3 个一组
    const securityChunks = chunkArray(formData.securities, 5);
    const structureChunks = chunkArray(formData.structures, 3);
    
    const totalChunks = securityChunks.length * structureChunks.length;
    let completedChunks = 0;
    let isFirstRequest = true;

    for (const sChunk of securityChunks) {
      // 获取当前正在处理的标的名称列表
      const targetNames = sChunk.map(s => s.securityName).join(', ');
      
      for (const stChunk of structureChunks) {
        currentInquiryTarget.value = `正在提交: ${targetNames}...`;
        
        // 如果不是第一个请求，则添加随机延迟（1~2.5秒）
        if (!isFirstRequest) {
          const delay = Math.floor(Math.random() * (2500 - 1000 + 1)) + 1000;
          console.log(`Waiting ${delay}ms before next request...`);
          await sleep(delay);
        }
        isFirstRequest = false;

        const payload = {
          securities: sChunk.map(s => ({
            securityCode: s.securityCode,
            securityName: s.securityName
          })),
          notionalPrincipal: notional,
          dateLimits: formData.dateLimits,
          structures: stChunk,
          specialRequirements: formData.specialRequirements
        };
        
        const response: any = await invoke("create_inquiry", { 
          body: JSON.stringify(payload),
          token: token.value
        });
        
        completedChunks++;
        inquiryProgress.value = Math.floor((completedChunks / totalChunks) * 100);

        if (response.success && response.data && response.data.success) {
          if (response.data.data && response.data.data.msgType === "A_STOCK_AFTERNOON_STOP") {
            lastError = "提示：沪深A股交易时间已结束，当前平台已停止询价。";
            failCount++;
          } else {
            successCount++;
          }
        } else {
          const pythonResult = response.data;
          if (pythonResult?.status_code === 401 || (pythonResult?.error && pythonResult?.error.includes("401"))) {
            isTokenExpired.value = true;
            tokenExpiryMessage.value = "Token 已过期或无效，请在主页更新。";
          }
          lastError = pythonResult?.msg || response.error || pythonResult?.error || "提交失败";
          failCount++;
        }
      }
    }
    
    if (successCount > 0) {
      showInquirySuccess.value = true;
      currentInquiryTarget.value = failCount === 0 ? "全部询价申请提交成功！" : `部分提交成功 (${successCount} 成功, ${failCount} 失败)`;
      
      // 清空已选标的
      formData.securities = [];
      
      // Refresh list
      fetchHistory();
      
      // 3秒后隐藏进度条成功状态
      setTimeout(() => {
        if (!createLoading.value) {
          showInquirySuccess.value = false;
          inquiryProgress.value = 0;
          currentInquiryTarget.value = "";
        }
      }, 5000);
    } else {
      alert("提交询价失败: " + lastError);
    }
  } catch (error) {
    console.error("Create inquiry failed:", error);
    alert("提交询价时发生系统错误");
  } finally {
    createLoading.value = false;
  }
};

const getBestQuote = (quoteItems: any[]) => {
  if (!quoteItems || quoteItems.length === 0) return null;
  const minPrice = Math.min(...quoteItems.map((q: any) => q.price));
  return quoteItems.find((q: any) => q.price === minPrice);
};

const getTValue = (createTimeStr: string, earliestDateStr: string) => {
  if (!createTimeStr || !earliestDateStr) return 'T+5';
  
  try {
    const createDate = new Date(createTimeStr.split(' ')[0]);
    const earliestDate = new Date(earliestDateStr.split(' ')[0]);
    
    createDate.setHours(0, 0, 0, 0);
    earliestDate.setHours(0, 0, 0, 0);
    
    const diffTime = earliestDate.getTime() - createDate.getTime();
    const diffDays = Math.round(diffTime / (1000 * 60 * 60 * 24));
    
    return diffDays <= 3 ? 'T+1' : 'T+5';
  } catch (e) {
    return 'T+5';
  }
};

const copyStatus = ref<Record<string, boolean>>({});
const markupValues = ref<Record<string, number | null>>({});
const tOneEnabled = ref<Record<string, boolean>>({});

const getMarkup = (id: string) => {
  const val = markupValues.value[id];
  // 逻辑：如果个股有单独设置（数字类型且不是 NaN），则使用个股设置
  if (typeof val === 'number' && !isNaN(val)) {
    return val;
  }
  // 否则，使用全局加价，全局为空则为 0
  const global = globalMarkup.value;
  if (typeof global === 'number' && !isNaN(global)) {
    return global;
  }
  return 0;
};
const handleUpdateToken = (newToken: string) => {
  token.value = newToken.trim();
  isTokenExpired.value = false;
  tokenExpiryMessage.value = "";
  // 可以在此处保存到 localStorage 或后端
  fetchHistory(); // 更新后尝试刷新
};

const getGroupedInquiryItems = (inquiryItems: any[]) => {
  const groups: Record<string, any[]> = {};
  inquiryItems.forEach(item => {
    if (!groups[item.structureName]) {
      groups[item.structureName] = [];
    }
    groups[item.structureName].push(item);
  });
  // 保持结构名称排序，如 S100, S103...
  return Object.keys(groups).sort().map(name => ({
    name,
    items: groups[name]
  }));
};

const formatQuoteText = (item: any, id: string) => {
  const markup = getMarkup(id);
  // 1. 处理交易所后缀
  const getSuffix = (code: string) => {
    if (code.startsWith('60') || code.startsWith('68')) return '.SH';
    if (code.startsWith('00') || code.startsWith('30')) return '.SZ';
    return '';
  };
  const suffix = getSuffix(item.securityCode);
  const fullCode = `${item.securityCode}${suffix}`;
  
  // 2. 标题居中 (简单通过空格模拟)
  const title = `${fullCode} ${item.securityName}`;
  const centeredTitle = `      ${title}     `;
  
  let text = `${centeredTitle}\n\n`;
  
  const allQuotes: any[] = [];
  item.inquiryItems.forEach((inquiry: any) => {
    const bestQuote = getBestQuote(inquiry.quoteItems);
    if (bestQuote) {
      // 3. 期限格式转换 (M1 -> 1M, W2 -> 2W)
      let displayLimit = inquiry.dateLimit;
      if (displayLimit.startsWith('M') || displayLimit.startsWith('W')) {
        displayLimit = displayLimit.substring(1) + displayLimit.charAt(0);
      }

      allQuotes.push({
        structureName: inquiry.structureName,
        dateLimit: displayLimit,
        price: (bestQuote.price + markup).toFixed(2)
      });
    }
  });

  allQuotes.forEach((quote, index) => {
    // 提取结构名称中的百分数，如 "S100" -> "100", "S103" -> "103"
    const structureNum = quote.structureName.replace(/[^0-9]/g, '');
    text += `${index + 1}: 【${structureNum}C，${quote.dateLimit}，${quote.price}%】\n`;
  });

  const isMainBoard = item.securityCode.startsWith('60') || item.securityCode.startsWith('00');
  const knockoutRule = isMainBoard ? '連3板敲出' : '連2板敲出';

  // 根据 tOneEnabled 状态决定显示 T+1 还是 T+5，默认为 T+5
  const tValue = tOneEnabled.value[id] ? 'T+1' : 'T+5';

  text += `\n盤中股價漲跌5%以上本報價無效\n`;
  text += `敲出規則：${knockoutRule}\n`;
  text += `最快時間：${tValue}`;

  return text;
};

const handleCopy = async (item: any, id: string) => {
  const text = formatQuoteText(item, id);
  try {
    await navigator.clipboard.writeText(text);
    copyStatus.value[id] = true;
    setTimeout(() => {
      copyStatus.value[id] = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy text: ', err);
  }
};

</script>

<template>
  <div class="flex h-screen bg-[#0f172a] text-slate-200 overflow-hidden font-sans">
    <!-- Sidebar -->
    <aside class="w-64 bg-[#1e293b] border-r border-slate-700 flex flex-col">
      <div class="p-6">
        <div class="flex items-center gap-3 mb-2">
          <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center shadow-lg shadow-blue-500/20">
            <RefreshCw class="w-6 h-6 text-white" />
          </div>
          <h1 class="text-xl font-bold tracking-tight text-white">Astin Inquiry</h1>
        </div>
        
        <!-- Version info under title -->
        <div class="flex items-center gap-2 mb-8 ml-[52px]">
          <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-500 border border-slate-700 font-medium">v0.1.0 Alpha</span>
          <div class="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse"></div>
        </div>

        <nav class="space-y-1">
          <button 
            @click="currentTab = 'home'"
            :class="currentTab === 'home' ? 'bg-blue-600/10 text-blue-400 font-medium' : 'text-slate-400 hover:bg-slate-800 hover:text-slate-200'"
            class="w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-all group"
          >
            <Home class="w-5 h-5 group-hover:text-blue-400" />
            <span>应用主页</span>
          </button>
          <button 
            @click="currentTab = 'inquiry'"
            :class="currentTab === 'inquiry' ? 'bg-blue-600/10 text-blue-400 font-medium' : 'text-slate-400 hover:bg-slate-800 hover:text-slate-200'"
            class="w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-all group"
          >
            <LayoutDashboard class="w-5 h-5 group-hover:text-blue-400" />
            <span>实时询价</span>
          </button>
        </nav>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col overflow-hidden">
      <!-- Header -->
      <header class="h-16 border-b border-slate-800 flex items-center justify-between px-8 bg-[#0f172a]/80 backdrop-blur-md z-10">
        <div class="flex items-center gap-4">
          <h2 class="text-lg font-semibold text-slate-100">
            {{ currentTab === 'home' ? '应用主页' : '实时询价' }}
          </h2>
        </div>
        <div class="flex items-center gap-4">
          <div class="w-8 h-8 rounded-full bg-slate-700 border border-slate-600 flex items-center justify-center text-[10px] font-bold text-slate-400">
            AD
          </div>
        </div>
      </header>

      <!-- Scrollable Content -->
      <div class="flex-1 overflow-y-auto p-8 custom-scrollbar">
        <!-- Home Tab -->
        <div v-if="currentTab === 'home'" class="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
          <!-- Auth Status Card -->
          <div class="bg-slate-800/40 border border-slate-700/50 rounded-3xl p-8 backdrop-blur-sm relative overflow-hidden">
            <div class="absolute -right-8 -top-8 opacity-5">
              <ShieldCheck v-if="!isTokenExpired" class="w-48 h-48 text-green-500" />
              <ShieldAlert v-else class="w-48 h-48 text-red-500" />
            </div>

            <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 relative z-10">
              <div class="space-y-2">
                <div class="flex items-center gap-3">
                  <div :class="!isTokenExpired ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'" class="w-12 h-12 rounded-2xl flex items-center justify-center">
                    <ShieldCheck v-if="!isTokenExpired" class="w-7 h-7" />
                    <ShieldAlert v-else class="w-7 h-7" />
                  </div>
                  <div>
                    <h3 class="text-xl font-bold text-white">身份验证状态</h3>
                    <p :class="[!isTokenExpired ? 'text-green-400/80' : 'text-red-400/80', isTokenExpired ? 'animate-pulse' : '']" class="text-sm font-medium">
                      {{ !isTokenExpired ? '身份认证已通过，欢迎您的使用' : '身份认证已过期，请及时更新 Token' }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Token Update Card -->
          <div class="bg-slate-800/40 border border-slate-700/50 rounded-3xl p-8 backdrop-blur-sm">
            <div class="flex items-center gap-3 mb-6">
              <div class="w-10 h-10 bg-slate-700/50 rounded-xl flex items-center justify-center text-blue-400">
                <KeyRound class="w-6 h-6" />
              </div>
              <h3 class="text-lg font-bold text-white">Token 认证更新</h3>
            </div>
            
            <div class="space-y-4">
              <div class="relative">
                <textarea 
                  v-model="token"
                  placeholder="在此处粘贴新的 Authorization Token..." 
                  rows="4"
                  class="w-full bg-slate-900/50 border border-slate-700 rounded-2xl p-4 text-slate-200 font-mono text-sm focus:outline-none focus:border-blue-500/50 focus:ring-4 focus:ring-blue-500/10 transition-all resize-none"
                ></textarea>
                <div v-if="token" class="absolute right-4 bottom-4 flex items-center gap-2">
                  <span class="text-[10px] text-slate-500 uppercase font-bold tracking-widest">Token 已输入</span>
                  <div class="w-2 h-2 bg-blue-500 rounded-full"></div>
                </div>
              </div>
              <div class="flex items-center justify-between gap-4">
                <p class="text-xs text-slate-500 max-w-md">提示：请确保填写的 Token 包含 "Bearer " 前缀（如果接口需要）。系统会自动修剪首尾空格。</p>
                <button @click="handleUpdateToken(token)" class="px-8 py-3 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-2xl font-bold transition-all border border-slate-600 active:scale-95">
                  保存并应用
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Inquiry Tab -->
        <div v-if="currentTab === 'inquiry'" class="space-y-8 animate-in fade-in slide-in-from-right-4 duration-500">
          <section class="max-w-[1440px] mx-auto w-full">
            <div class="bg-gradient-to-br from-[#1e293b] to-[#0f172a] rounded-3xl p-10 border border-slate-700/50 shadow-2xl relative overflow-hidden group">
              <div class="absolute top-0 right-0 p-10 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                <Plus class="w-48 h-48" />
              </div>
              
              <div class="relative z-10 space-y-10">
                <div class="flex items-center gap-4">
                  <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center shadow-lg shadow-blue-500/20">
                    <Plus class="w-6 h-6 text-white" />
                  </div>
                  <div>
                    <h3 class="text-2xl font-bold text-white">新建询价</h3>
                    <p class="text-slate-400 text-sm">配置结构化询价参数并提交申请</p>
                  </div>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-12">
                  <!-- Left Column: Securities & Structures -->
                  <div class="space-y-8">
                    <!-- 询价标的 -->
                    <div class="space-y-4 relative">
                      <label class="flex items-center gap-2 text-sm font-bold text-slate-400 uppercase tracking-widest">
                        <Search class="w-4 h-4" />
                        询价标的
                      </label>
                      <div class="relative group">
                        <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500 group-focus-within:text-blue-400 transition-colors" />
                        <input 
                          v-model="query"
                          @input="handleInputSearch"
                          type="text" 
                          placeholder="输入代码或名称添加标的..." 
                          class="w-full bg-slate-800/50 border border-slate-700 rounded-2xl py-4 pl-12 pr-4 text-slate-200 focus:outline-none focus:border-blue-500/50 focus:ring-4 focus:ring-blue-500/10 transition-all shadow-inner"
                        >
                        
                        <!-- Dropdown Preview -->
                        <div v-if="showDropdown" class="absolute left-0 right-0 mt-2 bg-slate-800 border border-slate-700 rounded-xl shadow-2xl z-50 overflow-hidden backdrop-blur-xl">
                          <div v-if="securityPool.length > 0" class="max-h-60 overflow-y-auto custom-scrollbar">
                            <div 
                              v-for="item in securityPool" 
                              :key="item.securityCode"
                              @click="selectSecurity(item)"
                              class="px-4 py-3 hover:bg-blue-600/20 cursor-pointer flex items-center justify-between group/item transition-colors"
                            >
                              <div class="flex items-center gap-3">
                                <span class="text-blue-400 font-mono font-bold">{{ item.securityCode }}</span>
                                <span class="text-slate-200">{{ item.securityName }}</span>
                              </div>
                              <Plus class="w-4 h-4 text-slate-600 group-hover/item:text-blue-400 transition-colors" />
                            </div>
                          </div>
                          <div v-else class="p-8 text-center text-slate-500">
                            <Search class="w-8 h-8 mx-auto mb-2 opacity-20" />
                            <p class="text-sm">未找到相关标的</p>
                          </div>
                        </div>
                      </div>
                      
                      <!-- Selected Securities Tags -->
                      <div class="flex flex-wrap gap-2">
                        <div 
                          v-for="s in formData.securities" 
                          :key="s.securityCode"
                          class="flex items-center gap-2 px-3 py-1.5 bg-blue-600/10 border border-blue-500/30 rounded-lg group animate-in fade-in zoom-in duration-200"
                        >
                          <span class="text-xs font-bold text-blue-400">{{ s.securityCode }}</span>
                          <span class="text-xs text-slate-300">{{ s.securityName }}</span>
                          <button @click="removeSecurity(s.securityCode)" class="text-slate-500 hover:text-red-400 transition-colors">
                            <X class="w-3 h-3" />
                          </button>
                        </div>
                      </div>
                    </div>

                    <!-- 询价结构 -->
                    <div class="space-y-4">
                      <label class="text-sm font-bold text-slate-400 uppercase tracking-widest block">询价结构</label>
                      <div class="space-y-3">
                        <div v-for="(row, rowIndex) in structureRows" :key="rowIndex" class="flex flex-wrap gap-3">
                          <button 
                            v-for="opt in row" :key="opt"
                            @click="toggleSelection(formData.structures, opt)"
                            :class="formData.structures.includes(opt) ? 'bg-blue-600 text-white border-blue-500 shadow-lg shadow-blue-600/20' : 'bg-slate-800/50 text-slate-400 border-slate-700 hover:border-slate-500'"
                            class="px-8 py-3 rounded-2xl border text-base font-bold transition-all active:scale-95"
                          >
                            {{ opt }}
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- Right Column: Scale, Tenor, Requirements -->
                  <div class="space-y-8">
                    <!-- 询价规模 -->
                    <div class="space-y-4">
                      <label class="text-sm font-bold text-slate-400 uppercase tracking-widest block">询价规模</label>
                      <div class="space-y-4">
                        <!-- Common Options Row -->
                        <div class="flex items-center gap-6">
                          <label v-for="opt in scaleOptions.filter(o => o !== 'custom')" :key="opt" class="flex items-center gap-3 cursor-pointer group">
                            <div class="relative flex items-center justify-center">
                              <input 
                                type="radio" 
                                v-model="formData.notionalPrincipal" 
                                :value="opt"
                                class="peer w-5 h-5 opacity-0 absolute cursor-pointer"
                              >
                              <div class="w-5 h-5 border-2 border-slate-700 rounded-full peer-checked:border-blue-500 peer-checked:bg-blue-600 transition-all shadow-inner"></div>
                              <div class="absolute w-2 h-2 bg-white rounded-full opacity-0 peer-checked:opacity-100 transition-opacity"></div>
                            </div>
                            <span class="text-base font-medium transition-colors group-hover:text-blue-400" :class="formData.notionalPrincipal === opt ? 'text-blue-400' : 'text-slate-400'">
                              {{ opt }}万
                            </span>
                          </label>
                        </div>

                        <!-- Custom Option Row -->
                        <div class="flex items-center gap-6">
                          <label class="flex items-center gap-3 cursor-pointer group">
                            <div class="relative flex items-center justify-center">
                              <input 
                                type="radio" 
                                v-model="formData.notionalPrincipal" 
                                value="custom"
                                class="peer w-5 h-5 opacity-0 absolute cursor-pointer"
                              >
                              <div class="w-5 h-5 border-2 border-slate-700 rounded-full peer-checked:border-blue-500 peer-checked:bg-blue-600 transition-all shadow-inner"></div>
                              <div class="absolute w-2 h-2 bg-white rounded-full opacity-0 peer-checked:opacity-100 transition-opacity"></div>
                            </div>
                            <span class="text-base font-medium transition-colors group-hover:text-blue-400" :class="formData.notionalPrincipal === 'custom' ? 'text-blue-400' : 'text-slate-400'">
                              自定义
                            </span>
                          </label>
                          
                          <div v-if="formData.notionalPrincipal === 'custom'" class="relative animate-in fade-in slide-in-from-left-4 duration-300">
                            <input 
                              v-model.number="formData.customNotional" 
                              type="number" 
                              placeholder="金额" 
                              class="bg-slate-900/50 border border-slate-700 rounded-xl py-2 px-4 text-base text-slate-100 focus:ring-2 focus:ring-blue-500 outline-none w-36 shadow-inner"
                            >
                            <span class="absolute right-4 top-1/2 -translate-y-1/2 text-sm text-slate-500 font-bold">万</span>
                          </div>
                        </div>
                      </div>
                    </div>

                    <!-- 期限 -->
                    <div class="space-y-4">
                      <label class="text-sm font-bold text-slate-400 uppercase tracking-widest block">询价期限</label>
                      <div class="space-y-3">
                        <div v-for="(row, rowIndex) in tenorRows" :key="rowIndex" class="flex flex-wrap gap-x-6 gap-y-3">
                          <label v-for="opt in row" :key="opt.value" class="flex items-center gap-3 cursor-pointer group">
                            <div class="relative flex items-center justify-center">
                              <input 
                                type="checkbox" 
                                :value="opt.value"
                                v-model="formData.dateLimits"
                                class="peer w-5 h-5 opacity-0 absolute cursor-pointer"
                              >
                              <div class="w-5 h-5 border-2 border-slate-700 rounded-lg peer-checked:border-blue-500 peer-checked:bg-blue-600 transition-all shadow-inner"></div>
                              <Check class="absolute w-3.5 h-3.5 text-white opacity-0 peer-checked:opacity-100 transition-opacity" />
                            </div>
                            <span class="text-base font-medium transition-colors group-hover:text-blue-400" :class="formData.dateLimits.includes(opt.value) ? 'text-blue-400' : 'text-slate-400'">
                              {{ opt.label }}
                            </span>
                          </label>
                        </div>
                      </div>
                    </div>

                    <!-- 特殊要求 -->
                    <div class="space-y-4">
                      <label class="text-sm font-bold text-slate-400 uppercase tracking-widest block">特殊要求</label>
                      <div class="flex flex-wrap gap-x-8 gap-y-4">
                        <label v-for="opt in specialOptions" :key="opt.value" class="flex items-center gap-3 cursor-pointer group">
                          <div class="relative flex items-center justify-center">
                            <input 
                              type="checkbox" 
                              :value="opt.value"
                              v-model="formData.specialRequirements"
                              class="peer w-5 h-5 opacity-0 absolute cursor-pointer"
                            >
                            <div class="w-5 h-5 border-2 border-slate-700 rounded-lg peer-checked:border-blue-500 peer-checked:bg-blue-600 transition-all shadow-inner"></div>
                            <Check class="absolute w-3.5 h-3.5 text-white opacity-0 peer-checked:opacity-100 transition-opacity" />
                          </div>
                          <div class="flex items-center gap-2">
                            <span class="text-base font-medium transition-colors group-hover:text-blue-400" :class="formData.specialRequirements.includes(opt.value) ? 'text-blue-400' : 'text-slate-400'">
                              {{ opt.label }}
                            </span>
                            <Info v-if="opt.value === 'tripleKnockout'" class="w-4 h-4 text-slate-600 group-hover:text-blue-500/50 transition-colors" />
                          </div>
                        </label>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Submit Button & Progress -->
                <div class="pt-6 border-t border-slate-700/50 flex items-center justify-between gap-8">
                  <!-- Progress UI -->
                  <div v-if="createLoading || showInquirySuccess" class="flex-1 max-w-2xl animate-in fade-in slide-in-from-left-4 duration-500">
                    <div class="flex items-center justify-between mb-2.5">
                      <div class="flex items-center gap-3 overflow-hidden">
                        <div v-if="createLoading" class="w-2 h-2 rounded-full bg-blue-500 animate-pulse shrink-0"></div>
                        <Check v-else class="w-4 h-4 text-green-500 shrink-0" />
                        <span class="text-sm font-bold text-slate-200 truncate">
                          {{ currentInquiryTarget }}
                        </span>
                      </div>
                      <span class="text-sm font-black text-blue-400 tabular-nums">{{ inquiryProgress }}%</span>
                    </div>
                    <div class="h-3 w-full bg-slate-900/80 rounded-full overflow-hidden border border-slate-700/50 p-0.5 shadow-inner">
                      <div 
                        class="h-full rounded-full bg-gradient-to-r from-blue-600 via-blue-500 to-indigo-500 transition-all duration-500 ease-out relative shadow-lg"
                        :style="{ width: `${inquiryProgress}%` }"
                      >
                        <!-- Shimmer effect -->
                        <div class="absolute inset-0 bg-[length:40px_40px] bg-[linear-gradient(45deg,rgba(255,255,255,0.15)_25%,transparent_25%,transparent_50%,rgba(255,255,255,0.15)_50%,rgba(255,255,255,0.15)_75%,transparent_75%,transparent)] animate-[shimmer_2s_infinite]"></div>
                      </div>
                    </div>
                    <div v-if="showInquirySuccess" class="mt-2 flex items-center gap-2 text-xs font-bold text-green-400 animate-in fade-in slide-in-from-top-2">
                      <div class="w-1.5 h-1.5 rounded-full bg-green-500"></div>
                      <span>所有标的询价已成功推送到队列</span>
                    </div>
                  </div>
                  <div v-else class="flex-1"></div>

                  <div class="flex flex-col items-end gap-2 shrink-0">
                    <button 
                      @click="handleCreate"
                      :disabled="createLoading || formData.securities.length === 0 || formData.dateLimits.length === 0 || formData.structures.length === 0"
                      class="bg-blue-600 hover:bg-blue-500 disabled:opacity-30 disabled:grayscale text-white px-12 py-4 rounded-2xl font-black text-lg flex items-center gap-3 transition-all shadow-2xl shadow-blue-600/40 active:scale-95 hover:-translate-y-0.5"
                    >
                      <Loader2 v-if="createLoading" class="w-6 h-6 animate-spin" />
                      <Send v-else class="w-5 h-5" />
                      <span>{{ createLoading ? '正在批量提交...' : '立即开始询价' }}</span>
                    </button>
                    <p v-if="formData.securities.length === 0" class="text-[10px] text-amber-500 font-medium">请先从搜索结果中选择标的</p>
                    <p v-else-if="formData.dateLimits.length === 0 || formData.structures.length === 0" class="text-[10px] text-amber-500 font-medium">请选择询价期限和结构</p>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- History Section -->
          <section class="max-w-[1440px] mx-auto w-full pb-12">
            <div class="flex items-end justify-between mb-6 pt-8 border-t border-slate-800/50">
              <div class="flex items-end gap-6">
                <div class="space-y-1">
                  <h3 class="text-xl font-bold text-white flex items-center gap-2">
                    <History class="w-5 h-5 text-blue-400" />
                    最近询价记录
                  </h3>
                  <p class="text-xs text-slate-500">共 {{ history.length }} 条原始记录，当前显示 {{ filteredHistory.length }} 条</p>
                </div>

                <!-- Refresh Controls Side by Side with Title -->
                <div class="flex items-center gap-4 pb-1">
                  <!-- Auto Refresh Switch -->
                  <div class="flex items-center gap-2 bg-slate-800/50 px-3 py-1.5 rounded-lg border border-slate-700">
                    <div 
                      @click="isAutoRefreshing = !isAutoRefreshing; if(isAutoRefreshing) startAutoRefresh()"
                      class="relative w-8 h-4 rounded-full cursor-pointer transition-colors duration-200"
                      :class="isAutoRefreshing ? 'bg-blue-600' : 'bg-slate-600'"
                    >
                      <div 
                        class="absolute top-0.5 left-0.5 w-3 h-3 bg-white rounded-full transition-transform duration-200"
                        :class="isAutoRefreshing ? 'translate-x-4' : 'translate-x-0'"
                      ></div>
                    </div>
                    <span class="text-[10px] font-bold uppercase tracking-wider transition-colors" :class="isAutoRefreshing ? 'text-blue-400' : 'text-slate-500'">
                      {{ isAutoRefreshing ? 'Auto Sync' : 'Manual' }}
                    </span>
                  </div>

                  <!-- Manual Refresh & Time -->
                  <div class="flex items-center gap-2 text-slate-500">
                    <button 
                      @click="fetchHistory()" 
                      class="p-1.5 hover:text-white hover:bg-slate-800 rounded-md transition-all group"
                      title="立即刷新"
                    >
                      <RefreshCw class="w-4 h-4 group-active:rotate-180 transition-transform duration-500" :class="{'animate-spin': loading}" />
                    </button>
                    <div class="flex flex-col">
                      <span class="text-[9px] uppercase font-bold leading-none opacity-50">Last Update</span>
                      <span class="text-[11px] tabular-nums font-medium">{{ lastRefreshTime.toLocaleTimeString() }}</span>
                    </div>
                  </div>
                </div>
              </div>
              
              <div class="flex items-center gap-4">
                <!-- Global Markup Control -->
                <div class="flex items-center bg-slate-800/50 border border-slate-700 rounded-xl px-3 py-1.5 transition-all hover:border-blue-500/30">
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-bold text-slate-500 uppercase tracking-widest">全局加价</span>
                    <div class="flex items-center bg-slate-900 border border-slate-700 rounded-md px-2 py-0.5">
                      <span class="text-[10px] font-bold text-slate-500 mr-1">+【</span>
                      <input 
                        v-model.number="globalMarkup" 
                        type="number" 
                        step="0.1"
                        placeholder="1.0"
                        class="w-12 bg-transparent border-none outline-none text-xs font-bold text-blue-400 text-center [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                      >
                      <span class="text-[10px] font-bold text-slate-500 ml-1">】%</span>
                    </div>
                  </div>
                </div>

                <div class="relative group">
                  <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500 group-focus-within:text-blue-400 transition-colors" />
                  <input 
                    v-model="historySearchQuery"
                    type="text" 
                    placeholder="搜索历史记录 (代码/名称)..." 
                    class="w-64 bg-slate-800/50 border border-slate-700 rounded-xl py-2 pl-10 pr-4 text-sm focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all"
                  >
                </div>
              </div>
            </div>

            <div class="bg-[#1e293b] rounded-2xl border border-slate-700/50 shadow-xl overflow-hidden">
              <table class="w-full text-left table-fixed">
                <thead>
                  <tr class="bg-slate-800/50 border-b border-slate-700">
                    <th class="w-[210px] px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider">询价时间</th>
                    <th class="w-[110px] px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider">标的代码</th>
                    <th class="w-[160px] px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider">标的名称</th>
                    <th class="w-[150px] px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider">名义本金</th>
                    <th class="px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider">报价项</th>
                    <th class="w-[90px] px-6 py-4 text-xs font-semibold text-slate-400 uppercase tracking-wider text-right">操作</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800">
                  <template v-for="(item, index) in filteredHistory" :key="getRowId(item, index)">
                    <tr 
                      @click="toggleRow(getRowId(item, index))"
                      class="hover:bg-slate-800/30 transition-colors group cursor-pointer"
                      :class="{'bg-blue-600/5': expandedRows.includes(getRowId(item, index))}"
                    >
                      <td class="px-6 py-4 text-sm text-slate-400 truncate">
                        <div class="flex items-center gap-2">
                          <ChevronRight 
                            class="w-4 h-4 text-slate-500 flex-shrink-0 transition-transform"
                            :class="{'rotate-90 text-blue-400': expandedRows.includes(getRowId(item, index))}"
                          />
                          <span class="truncate">{{ item.createTime }}</span>
                        </div>
                      </td>
                      <td class="px-6 py-4 text-sm font-bold text-blue-400 truncate">{{ item.securityCode }}</td>
                      <td class="px-6 py-4 text-sm font-medium text-slate-200 truncate whitespace-nowrap">{{ item.securityName }}</td>
                      <td class="px-6 py-4 text-sm text-slate-300 truncate">¥{{ item.notionalValue.toLocaleString() }}</td>
                      <td class="px-6 py-4">
                        <div class="flex gap-4">
                          <div v-for="group in getGroupedInquiryItems(item.inquiryItems).slice(0, 3)" :key="group.name" class="flex flex-col gap-1">
                            <span v-for="inquiry in group.items" :key="inquiry.optInquiryBuyerId" 
                              class="px-2 py-0.5 rounded text-[10px] bg-slate-700 text-slate-300 border border-slate-600 whitespace-nowrap flex items-center gap-1.5 w-32">
                              <span class="opacity-60 flex-1">{{ inquiry.structureName }}/{{ inquiry.dateLimit }}</span>
                              <span v-if="getBestQuote(inquiry.quoteItems)" class="text-green-400 font-bold border-l border-slate-600 pl-1.5">
                                {{ getBestQuote(inquiry.quoteItems).price.toFixed(2) }}%
                              </span>
                            </span>
                          </div>
                          <!-- More indicator if structures > 3 -->
                          <div v-if="getGroupedInquiryItems(item.inquiryItems).length > 3" class="flex items-center">
                            <span class="text-[10px] text-slate-500 font-bold bg-slate-800/50 px-2 py-1 rounded border border-slate-700/50">
                              +{{ getGroupedInquiryItems(item.inquiryItems).length - 3 }} 更多结构
                            </span>
                          </div>
                        </div>
                      </td>
                      <td class="px-6 py-4 text-right">
                        <div class="flex items-center justify-end gap-2">
                          <button 
                            @click.stop="handleCopy(item, getRowId(item, index))"
                            class="p-1.5 rounded-lg transition-all"
                            :class="copyStatus[getRowId(item, index)] ? 'text-green-400 bg-green-400/10' : 'text-slate-400 hover:text-blue-400 hover:bg-slate-700'"
                            :title="copyStatus[getRowId(item, index)] ? '已复制' : '复制报价文本'"
                          >
                            <Check v-if="copyStatus[getRowId(item, index)]" class="w-4 h-4" />
                            <Copy v-else class="w-4 h-4" />
                          </button>
                          <button class="p-1.5 rounded-lg text-slate-400 hover:text-blue-400 hover:bg-slate-700 transition-all">
                            <ExternalLink class="w-4 h-4" />
                          </button>
                        </div>
                      </td>
                    </tr>
                    <!-- Expanded Row Content -->
                    <tr v-if="expandedRows.includes(getRowId(item, index))" class="bg-slate-900/50">
                      <td colspan="6" class="px-12 py-8">
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                          <!-- Best Quote Cards (Left: 2/3 width) -->
                          <div class="lg:col-span-2 space-y-4">
                            <h5 class="text-xs font-bold text-slate-500 uppercase tracking-widest">最优报价明细</h5>
                            <div class="grid grid-cols-1 gap-4">
                              <div v-for="inquiry in item.inquiryItems" :key="inquiry.optInquiryBuyerId" 
                                class="bg-[#1e293b] rounded-2xl border border-slate-700/50 overflow-hidden shadow-lg transition-all hover:border-blue-500/30 group">
                                <div v-if="getBestQuote(inquiry.quoteItems)" class="p-5 flex items-center gap-6">
                                  <!-- Structure Info -->
                                  <div class="w-32 flex-shrink-0">
                                    <div class="text-xs font-bold text-blue-400 mb-1">{{ inquiry.structureName }}</div>
                                    <div class="text-lg font-black text-white">{{ inquiry.dateLimit }}</div>
                                  </div>

                                  <!-- Best Quote Price -->
                                  <div class="w-28 flex-shrink-0">
                                    <div class="text-[10px] text-slate-500 uppercase font-bold mb-1">最优价格</div>
                                    <div class="flex items-baseline gap-1">
                                      <span class="text-2xl font-black text-green-400">{{ getBestQuote(inquiry.quoteItems).price.toFixed(2) }}</span>
                                      <span class="text-xs font-bold text-slate-500">%</span>
                                    </div>
                                  </div>

                                  <!-- Estimated Fee -->
                                  <div class="w-36 flex-shrink-0">
                                    <div class="text-[10px] text-slate-500 uppercase font-bold mb-1">预估期权费</div>
                                    <div class="text-base font-bold text-slate-200">¥{{ ((item.notionalValue * getBestQuote(inquiry.quoteItems).price) / 100).toLocaleString() }}</div>
                                  </div>

                                  <!-- Seller Info -->
                                  <div class="flex-1">
                                    <div class="text-[10px] text-slate-500 uppercase font-bold mb-1">成交商</div>
                                    <div class="flex items-center gap-2">
                                      <span class="text-sm font-bold text-slate-300">交易员 #{{ getBestQuote(inquiry.quoteItems).sellerFundid }}</span>
                                      <span class="px-1.5 py-0.5 rounded bg-slate-800 text-[10px] text-slate-500 border border-slate-700">{{ getBestQuote(inquiry.quoteItems).busType }}</span>
                                    </div>
                                  </div>

                                  <!-- Best Badge -->
                                  <div class="flex-shrink-0 flex flex-col items-center gap-2">
                                    <div class="px-3 py-1 bg-green-500/10 text-green-400 border border-green-500/20 text-[10px] font-black rounded-lg uppercase tracking-wider">
                                      Best Choice
                                    </div>
                                    <div v-if="getBestQuote(inquiry.quoteItems).earliestExerciseDate" 
                                        :class="getTValue(item.createTime, getBestQuote(inquiry.quoteItems).earliestExerciseDate) === 'T+1' ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' : 'bg-blue-500/10 text-blue-400 border-blue-500/20'"
                                        class="px-2 py-0.5 border text-[10px] font-black rounded-md uppercase tracking-wider">
                                        最早可行权日: {{ getTValue(item.createTime, getBestQuote(inquiry.quoteItems).earliestExerciseDate) }}
                                      </div>
                                     <div v-if="getBestQuote(inquiry.quoteItems).refExpDate" 
                                        class="px-2 py-0.5 bg-slate-500/10 text-slate-400 border border-slate-500/20 text-[10px] font-black rounded-md uppercase tracking-wider">
                                        参考到期日: {{ getBestQuote(inquiry.quoteItems).refExpDate }}
                                      </div>
                                   </div>
                                </div>
                                <div v-else class="p-8 text-center bg-slate-800/20">
                                  <Loader2 class="w-5 h-5 animate-spin inline-block text-slate-600 mb-2" />
                                  <div class="text-xs text-slate-500 font-medium">正在等待最优报价...</div>
                                </div>
                              </div>
                            </div>
                          </div>

                          <!-- Formatted Quote Text (Right: 1/3 width) -->
                          <div class="lg:col-span-1 space-y-4">
                            <div class="flex items-center justify-between">
                              <div class="flex items-center gap-2">
                                <h5 class="text-xs font-bold text-slate-500 uppercase tracking-widest">格式化报价文本</h5>
                                <div class="flex items-center bg-slate-900 border border-slate-700 rounded-md px-1.5 py-0.5 ml-2">
                                  <span class="text-[10px] font-bold text-slate-500 mr-1">+【</span>
                                  <input 
                                    v-model.number="markupValues[getRowId(item, index)]" 
                                    type="number" 
                                    step="0.1"
                                    placeholder="0.0"
                                    class="w-10 bg-transparent border-none outline-none text-[10px] font-bold text-blue-400 text-center [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                                  >
                                  <span class="text-[10px] font-bold text-slate-500 ml-1">】%</span>
                                </div>
                              </div>
                              <div class="flex items-center gap-2">
                                <button 
                                  @click="tOneEnabled[getRowId(item, index)] = !tOneEnabled[getRowId(item, index)]"
                                  class="flex items-center gap-1.5 px-2 py-1 rounded-lg transition-all text-[10px] font-black uppercase tracking-wider border"
                                  :class="tOneEnabled[getRowId(item, index)] ? 'text-amber-400 bg-amber-400/10 border-amber-400/30' : 'text-slate-500 bg-slate-800/50 border-slate-700 hover:text-slate-400'"
                                >
                                  {{ tOneEnabled[getRowId(item, index)] ? 'T+1 已开启' : 'T+1 启用' }}
                                </button>
                                <button 
                                  @click.stop="handleCopy(item, getRowId(item, index))"
                                class="flex items-center gap-2 px-3 py-1 rounded-lg transition-all text-xs font-bold"
                                :class="copyStatus[getRowId(item, index)] ? 'text-green-400 bg-green-400/10' : 'text-blue-400 bg-blue-400/10 hover:bg-blue-400/20'"
                              >
                                <Check v-if="copyStatus[getRowId(item, index)]" class="w-3.5 h-3.5" />
                                <Copy v-else class="w-3.5 h-3.5" />
                                {{ copyStatus[getRowId(item, index)] ? '已复制' : '复制文本' }}
                               </button>
                             </div>
                           </div>
                            <div class="bg-slate-950/50 rounded-2xl p-6 border border-slate-800 shadow-inner group relative">
                              <pre class="text-xs font-mono text-slate-400 leading-relaxed whitespace-pre-wrap">{{ formatQuoteText(item, getRowId(item, index)) }}</pre>
                              <div class="absolute inset-0 bg-gradient-to-t from-slate-950/20 to-transparent pointer-events-none rounded-2xl"></div>
                            </div>
                          </div>
                        </div>
                      </td>
                    </tr>
                  </template>
                </tbody>
              </table>
              
              <!-- Pagination -->
              <div class="px-6 py-4 bg-slate-800/30 border-t border-slate-700 flex items-center justify-between">
                <span class="text-sm text-slate-500">共 {{ totalHistory }} 条记录</span>
                <div class="flex gap-2">
                  <button 
                    @click="currentPage--; fetchHistory()" 
                    :disabled="currentPage === 1"
                    class="p-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-400 disabled:opacity-30 hover:text-white transition-colors"
                  >
                    <ChevronRight class="w-4 h-4 rotate-180" />
                  </button>
                  <span class="px-4 py-2 text-sm text-slate-300 bg-slate-900 border border-slate-700 rounded-lg">{{ currentPage }}</span>
                  <button 
                    @click="currentPage++; fetchHistory()" 
                    :disabled="history.length < pageSize"
                    class="p-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-400 disabled:opacity-30 hover:text-white transition-colors"
                  >
                    <ChevronRight class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>
    </main>
  </div>
</template>

<style>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #334155;
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #475569;
}
</style>
