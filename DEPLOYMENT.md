# Deployment Guide - GridLens Energy Platform

## 🚀 Deploy to Render

### Prerequisites
- GitHub account with the repository
- Render account (free tier available)

### Step 1: Push to GitHub

1. **Create a new repository on GitHub**
   ```bash
   # Create a new repo on GitHub, then:
   git remote add origin https://github.com/yourusername/plentra-platform.git
   git branch -M main
   git push -u origin main
   ```

### Step 2: Deploy on Render

1. **Sign up/Login to Render**
   - Go to [render.com](https://render.com)
   - Sign up with your GitHub account

2. **Create New Web Service**
   - Click "New +" → "Web Service"
   - Connect your GitHub repository
   - Select the `plentra-platform` repository

3. **Configure the Service**
   - **Name**: `plentra-platform` (or your preferred name)
   - **Environment**: `Node`
   - **Build Command**: `npm install && npm run build`
   - **Start Command**: `npm run preview`
   - **Plan**: Free

4. **Environment Variables** (Optional)
   - `NODE_ENV`: `production`

5. **Deploy**
   - Click "Create Web Service"
   - Render will automatically build and deploy your app

### Step 3: Custom Domain (Optional)

1. **Add Custom Domain**
   - Go to your service settings
   - Click "Custom Domains"
   - Add your domain and configure DNS

### Step 4: Environment Variables

The app will work with default settings, but you can add:

```env
NODE_ENV=production
VITE_APP_TITLE=GridLens
VITE_APP_VERSION=2.1.4
```

## 🔧 Alternative Deployment Options

### Netlify
1. Connect GitHub repository
2. Build command: `npm run build`
3. Publish directory: `dist`
4. Deploy!

### Vercel
1. Import GitHub repository
2. Framework preset: Vite
3. Build command: `npm run build`
4. Output directory: `dist`
5. Deploy!

### GitHub Pages
1. Add to package.json:
   ```json
   "homepage": "https://yourusername.github.io/plentra-platform",
   "scripts": {
     "predeploy": "npm run build",
     "deploy": "gh-pages -d dist"
   }
   ```
2. Install gh-pages: `npm install --save-dev gh-pages`
3. Deploy: `npm run deploy`

## 📊 Performance Optimization

### Build Optimization
- The app is already optimized with Vite
- Tree-shaking removes unused code
- CSS is minified and optimized
- Images are optimized automatically

### Caching Strategy
- Static assets are cached aggressively
- API responses can be cached (when real APIs are added)
- Service worker for offline support (can be added)

## 🔍 Monitoring & Analytics

### Render Dashboard
- Monitor uptime and performance
- View logs and error tracking
- Set up alerts for downtime

### Analytics (Optional)
Add Google Analytics or similar:
```html
<!-- In index.html -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

## 🛠️ Troubleshooting

### Common Issues

1. **Build Fails**
   - Check Node.js version (use 18+)
   - Verify all dependencies are installed
   - Check for syntax errors in the code

2. **App Not Loading**
   - Verify the start command is correct
   - Check environment variables
   - Review Render logs for errors

3. **Styling Issues**
   - Ensure Tailwind CSS is building correctly
   - Check for CSS conflicts
   - Verify responsive breakpoints

### Debug Commands
```bash
# Local build test
npm run build

# Test production build locally
npm run preview

# Check for linting issues
npm run lint

# Analyze bundle size
npm run build -- --analyze
```

## 📈 Scaling

### Free Tier Limits
- 750 hours/month
- 512MB RAM
- Shared CPU

### Paid Plans
- More resources available
- Custom domains included
- Better performance
- Priority support

## 🔐 Security

### Best Practices
- Keep dependencies updated
- Use environment variables for secrets
- Enable HTTPS (automatic on Render)
- Regular security audits

### Environment Variables
Never commit sensitive data:
```bash
# .env (not committed)
API_KEY=your_secret_key
DATABASE_URL=your_db_url
```

## 📞 Support

- **Render Support**: [docs.render.com](https://docs.render.com)
- **Vite Documentation**: [vitejs.dev](https://vitejs.dev)
- **React Documentation**: [react.dev](https://react.dev)

---

**GridLens** - Ready for production deployment! 🚀
